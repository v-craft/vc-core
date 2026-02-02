#![allow(unused, reason = "todo")]

use alloc::boxed::Box;
use alloc::collections::VecDeque;
use alloc::vec::Vec;
use core::any::TypeId;
use core::ops::Range;

use vc_ptr::Ptr;
use vc_ptr::PtrMut;
use vc_reflect::Reflect;
use vc_utils::extra::PagePool;
use vc_utils::hash::SparseHashMap;
use vc_utils::hash::SparseHashSet;

use crate::bundle::InsertMode;
use crate::component::Component;
use crate::component::ComponentCloneBehavior;
use crate::component::ComponentCloneFn;
use crate::component::ComponentId;
use crate::component::ComponentInfo;
use crate::entity::Entity;
use crate::entity::EntityAllocator;
use crate::entity::EntityMapper;
use crate::reflect::AppTypeRegistry;
use crate::utils::DebugName;
use crate::world::World;

pub struct ComponentCloneCtx<'a, 'b> {
    component_id: ComponentId,
    target_component_written: bool,
    target_component_moved: bool,
    scratch_buffer: &'a mut ScratchBuffer<'b>,
    scratch_pool: &'b PagePool,
    source: Entity,
    target: Entity,
    allocator: &'a EntityAllocator,
    component_info: &'a ComponentInfo,
    state: &'a mut EntityClonerState,
    mapper: &'a mut dyn EntityMapper,
    type_registry: Option<&'a AppTypeRegistry>,
}

struct ScratchBuffer<'a> {
    component_ids: Vec<ComponentId>,
    component_ptrs: Vec<PtrMut<'a>>,
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

pub enum EntityClonerFilter {
    OptOut(OptOut),
    OptIn(OptIn),
}

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

impl<'a, 'b> ComponentCloneCtx<'a, 'b> {
    unsafe fn new(
        component_id: ComponentId,
        scratch_buffer: &'a mut ScratchBuffer<'b>,
        scratch_pool: &'b PagePool,
        source: Entity,
        target: Entity,
        allocator: &'a EntityAllocator,
        component_info: &'a ComponentInfo,
        state: &'a mut EntityClonerState,
        mapper: &'a mut dyn EntityMapper,
        type_registry: Option<&'a AppTypeRegistry>,
    ) -> Self {
        Self {
            component_id,
            target_component_written: false,
            target_component_moved: false,
            scratch_buffer,
            scratch_pool,
            source,
            target,
            allocator,
            component_info,
            state,
            mapper,
            type_registry,
        }
    }

    /// Returns the current source entity.
    pub fn source(&self) -> Entity {
        self.source
    }

    /// Returns the current target entity.
    pub fn target(&self) -> Entity {
        self.target
    }

    /// Returns the [`ComponentId`] of the component being cloned.
    pub fn component_id(&self) -> ComponentId {
        self.component_id
    }

    /// Returns the [`ComponentInfo`] of the component being cloned.
    pub fn component_info(&self) -> &ComponentInfo {
        self.component_info
    }

    /// Returns `true` if used in moving context
    pub fn moving(&self) -> bool {
        self.state.move_components
    }

    /// Returns true if `write_target_component` was called before.
    pub fn target_component_written(&self) -> bool {
        self.target_component_written
    }

    pub fn target_component_moved(&self) -> bool {
        self.target_component_moved
    }

    pub fn linked_cloning(&self) -> bool {
        self.state.linked_cloning
    }

    /// Returns this context's [`EntityMapper`].
    pub fn entity_mapper(&mut self) -> &mut dyn EntityMapper {
        self.mapper
    }

    pub fn type_registry(&self) -> Option<&AppTypeRegistry> {
        self.type_registry
    }

    fn move_component(&mut self) {
        self.target_component_moved = true;
        self.target_component_written = true;
    }

    pub fn queue_entity_clone(&mut self, entity: Entity) {
        let target = self.allocator.alloc();
        self.mapper.set_mapped(entity, target);
        self.state.clone_queue.push_back(entity);
    }

    pub fn queue_deferred(
        &mut self,
        deferred: impl FnOnce(&mut World, &mut dyn EntityMapper) + 'static,
    ) {
        self.state.deferred_commands.push_back(Box::new(deferred));
    }

    pub fn write_target_component<C: Component>(&mut self, mut component: C) {
        C::map_entities(&mut component, &mut self.mapper);
        let debug_name = DebugName::type_name::<C>();

        if self.target_component_written {
            panic!("Trying to write component '{debug_name}' multiple times");
        }

        if self.component_info.type_id() != Some(TypeId::of::<C>()) {
            panic!("TypeId of component '{debug_name}' does not match source component TypeId")
        };

        unsafe {
            self.scratch_buffer
                .push(self.scratch_pool, self.component_id, component);
        };
        self.target_component_written = true;
    }

    pub unsafe fn write_target_component_ptr(&mut self, ptr: Ptr) {
        if self.target_component_written {
            panic!("Trying to write component multiple times")
        }

        let layout = self.component_info.layout();
        let target_ptr = self.scratch_pool.alloc_layout(layout);
        unsafe {
            core::ptr::copy_nonoverlapping(ptr.as_ptr(), target_ptr.as_ptr(), layout.size());
            self.scratch_buffer
                .push_ptr(self.component_id, PtrMut::new(target_ptr));
        }
        self.target_component_written = true;
    }

    pub fn write_target_component_reflect(&mut self, component: Box<dyn Reflect>) {
        if self.target_component_written {
            panic!("Trying to write component multiple times")
        }
        let source_type_id = self
            .component_info
            .type_id()
            .expect("Source component must have TypeId");
        let component_type_id = (*component).type_id();
        if source_type_id != component_type_id {
            panic!("Passed component TypeId does not match source component TypeId")
        }

        let component_layout = self.component_info.layout();
        let source_data_ptr = Box::into_raw(component).cast::<u8>();
        let target_data_ptr = self.scratch_pool.alloc_layout(component_layout);
        unsafe {
            core::ptr::copy_nonoverlapping(
                source_data_ptr,
                target_data_ptr.as_ptr(),
                component_layout.size(),
            );
            self.scratch_buffer
                .push_ptr(self.component_id, PtrMut::new(target_data_ptr));

            if component_layout.size() > 0 {
                // Ensure we don't attempt to deallocate zero-sized components
                alloc::alloc::dealloc(source_data_ptr, component_layout);
            }
        }
    }
}

impl<'a> ScratchBuffer<'a> {
    pub(crate) fn with_capacity(capacity: usize) -> Self {
        Self {
            component_ids: Vec::with_capacity(capacity),
            component_ptrs: Vec::with_capacity(capacity),
        }
    }

    pub(crate) unsafe fn push_ptr(&mut self, id: ComponentId, ptr: PtrMut<'a>) {
        self.component_ids.push(id);
        self.component_ptrs.push(ptr);
    }

    pub(crate) unsafe fn push<C: Component>(
        &mut self,
        pool: &'a PagePool,
        id: ComponentId,
        component: C,
    ) {
        let component_mut = unsafe { pool.alloc_unchecked(component) };
        self.component_ids.push(id);
        self.component_ptrs.push(PtrMut::from_mut(component_mut));
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
