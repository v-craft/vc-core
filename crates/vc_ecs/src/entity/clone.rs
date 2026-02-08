#![allow(unused, reason = "todo")]

use alloc::boxed::Box;
use alloc::collections::VecDeque;
use alloc::vec::Vec;
use core::any::TypeId;
use core::ops::Range;
use core::ptr;

use vc_ptr::Ptr;
use vc_ptr::PtrMut;
use vc_reflect::Reflect;
use vc_utils::extra::PagePool;
use vc_utils::hash::SparseHashMap;
use vc_utils::hash::SparseHashSet;

use crate::component::{Component, ComponentId, ComponentInfo};
use crate::component::{ComponentCloneBehavior, ComponentCloneFn};
use crate::component::{InsertMode, SourceComponent};

use crate::entity::Entity;
use crate::entity::EntityAllocator;
use crate::entity::EntityMapper;
use crate::reflect::AppTypeRegistry;
use crate::utils::DebugName;
use crate::world::World;

// -----------------------------------------------------------------------------
// Types

pub struct ComponentCloneCtx<'a, 'b> {
    component_id: ComponentId,
    component_info: &'a ComponentInfo,
    source: Entity,
    target: Entity,
    scratch_pool: &'b PagePool,
    scratch_buffer: &'a mut ScratchBuffer<'b>,
    allocator: &'a EntityAllocator,
    state: &'a mut EntityClonerState,
    mapper: &'a mut dyn EntityMapper,
    target_component_written: bool,
    target_component_moved: bool,
    type_registry: Option<&'a AppTypeRegistry>,
}

struct ScratchBuffer<'a> {
    ids: Vec<ComponentId>,
    ptrs: Vec<PtrMut<'a>>,
}

struct EntityClonerState {
    clone_behavior_overrides: SparseHashMap<ComponentId, ComponentCloneBehavior>,
    move_components: bool,
    linked_cloning: bool,
    default_clone_fn: ComponentCloneFn,
    clone_queue: VecDeque<Entity>,
    deferred_commands: VecDeque<Box<dyn FnOnce(&mut World, &mut dyn EntityMapper)>>,
}

pub struct EntityCloner {
    filter: EntityClonerFilter,
    state: EntityClonerState,
}

pub struct EntityClonerBuilder<'w, Filter> {
    world: &'w mut World,
    filter: Filter,
    state: EntityClonerState,
}

pub enum EntityClonerFilter {
    OptOut(OptOut),
    OptIn(OptIn),
}

pub trait CloneByFilter: Into<EntityClonerFilter> {}

pub struct OptOut {
    deny: SparseHashSet<ComponentId>,
    insert_mode: InsertMode,
    attach_required_by_components: bool,
}

pub struct OptIn {
    allow: SparseHashMap<ComponentId, Explicit>,
    required_of_allow: Vec<ComponentId>,
    required: SparseHashMap<ComponentId, Required>,
    attach_required_components: bool,
}

struct Explicit {
    insert_mode: InsertMode,
    required_range: Option<Range<usize>>,
}

struct Required {
    required_by: u32,
    required_by_reduced: u32,
}

// -----------------------------------------------------------------------------
// ComponentCloneCtx Implementation

impl<'a, 'b> ComponentCloneCtx<'a, 'b> {
    /// Returns the [`ComponentId`] of the component being cloned.
    #[inline(always)]
    pub fn component_id(&self) -> ComponentId {
        self.component_id
    }

    /// Returns the [`ComponentInfo`] of the component being cloned.
    #[inline(always)]
    pub fn component_info(&self) -> &ComponentInfo {
        self.component_info
    }

    /// Returns the current source entity.
    #[inline(always)]
    pub fn source(&self) -> Entity {
        self.source
    }

    /// Returns the current target entity.
    #[inline(always)]
    pub fn target(&self) -> Entity {
        self.target
    }

    /// Returns `true` if used in moving context
    #[inline(always)]
    pub fn moving(&self) -> bool {
        self.state.move_components
    }

    #[inline(always)]
    pub fn target_component_written(&self) -> bool {
        self.target_component_written
    }

    #[inline(always)]
    pub fn target_component_moved(&self) -> bool {
        self.target_component_moved
    }

    /// Returns true if the [`EntityCloner`] is configured to recursively clone entities.
    #[inline(always)]
    pub fn linked_cloning(&self) -> bool {
        self.state.linked_cloning
    }

    /// Returns this context's [`EntityMapper`].
    #[inline(always)]
    pub fn entity_mapper(&mut self) -> &mut dyn EntityMapper {
        self.mapper
    }

    /// Returns this context's [`AppTypeRegistry`].
    #[inline(always)]
    pub fn type_registry(&self) -> Option<&AppTypeRegistry> {
        self.type_registry
    }

    /// Marks component as moved and it's `drop` won't run.
    #[inline(always)]
    fn move_component(&mut self) {
        self.target_component_moved = true;
        self.target_component_written = true;
    }

    #[inline]
    pub fn queue_entity_clone(&mut self, entity: Entity) {
        let target = self.allocator.alloc();
        self.mapper.set_mapped(entity, target);
        self.state.clone_queue.push_back(entity);
    }

    #[inline]
    pub fn queue_deferred(
        &mut self,
        deferred: impl FnOnce(&mut World, &mut dyn EntityMapper) + 'static,
    ) {
        self.state.deferred_commands.push_back(Box::new(deferred));
    }

    #[cold]
    #[inline(never)]
    fn handle_multiple_write(debug_name: &DebugName) -> ! {
        panic!("Trying to write component '{debug_name}' multiple times");
    }

    #[cold]
    #[inline(never)]
    fn handle_mismatched_type(debug_name: &DebugName) -> ! {
        panic!("TypeId of component '{debug_name}' does not match source component")
    }

    pub fn write_target_component<C: Component>(&mut self, mut component: C) {
        C::map_entities(&mut component, &mut self.mapper);

        if self.target_component_written {
            Self::handle_multiple_write(&DebugName::type_name::<C>());
        }

        if self.component_info.type_id() != Some(TypeId::of::<C>()) {
            Self::handle_mismatched_type(&DebugName::type_name::<C>());
        };

        unsafe {
            let component_mut = self.scratch_pool.alloc_unchecked(component);
            self.scratch_buffer.ids.push(self.component_id);
            self.scratch_buffer
                .ptrs
                .push(PtrMut::from_mut(component_mut));
        }

        self.target_component_written = true;
    }

    /// # Safety
    /// Caller must ensure that the passed in `ptr` references data
    /// that corresponds to the type of the source / target [`ComponentId`].
    pub unsafe fn write_target_component_ptr(&mut self, ptr: Ptr) {
        if self.target_component_written {
            Self::handle_multiple_write(&DebugName::anonymous());
        }

        let layout = self.component_info.layout();
        let target_ptr = self.scratch_pool.alloc_layout(layout);

        unsafe {
            ptr::copy_nonoverlapping(ptr.as_ptr(), target_ptr.as_ptr(), layout.size());
            self.scratch_buffer.ids.push(self.component_id);
            self.scratch_buffer.ptrs.push(PtrMut::new(target_ptr));
        }

        self.target_component_written = true;
    }

    pub fn write_target_component_reflect(&mut self, component: Box<dyn Reflect>) {
        if self.target_component_written {
            Self::handle_multiple_write(&DebugName::anonymous());
        }

        let source_type_id = self
            .component_info
            .type_id()
            .expect("Source component must have TypeId");

        if source_type_id != (*component).type_id() {
            Self::handle_mismatched_type(&DebugName::anonymous());
        }

        let layout = self.component_info.layout();
        let target_ptr = self.scratch_pool.alloc_layout(layout);

        unsafe {
            let source_ptr = Box::into_raw(component).cast::<u8>();
            ptr::copy_nonoverlapping(source_ptr, target_ptr.as_ptr(), layout.size());
            if layout.size() > 0 {
                // Ensure we don't attempt to deallocate zero-sized components
                alloc::alloc::dealloc(source_ptr, layout);
            }

            self.scratch_buffer.ids.push(self.component_id);
            self.scratch_buffer.ptrs.push(PtrMut::new(target_ptr));
        }
    }
}

// -----------------------------------------------------------------------------
// ScratchBuffer Implementation

impl<'a> ScratchBuffer<'a> {
    fn with_capacity(capacity: usize) -> Self {
        Self {
            ids: Vec::with_capacity(capacity),
            ptrs: Vec::with_capacity(capacity),
        }
    }

    // #[track_caller]
    // pub(crate) unsafe fn write(
    //     self,
    //     world: &mut World,
    //     entity: Entity,
    //     relationship_hook_insert_mode: RelationshipHookMode,
    // ) {
    //     // SAFETY:
    //     // - All `component_ids` are from the same world as `entity`
    //     // - All `component_data_ptrs` are valid types represented by `component_ids`
    //     unsafe {
    //         world.entity_mut(entity).insert_by_ids_internal(
    //             &self.component_ids,
    //             self.component_ptrs.into_iter().map(|ptr| ptr.promote()),
    //             relationship_hook_insert_mode,
    //         );
    //     }
    // }
}
