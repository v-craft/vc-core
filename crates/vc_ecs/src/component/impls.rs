#![allow(clippy::missing_safety_doc, reason = "todo")]

use alloc::vec::Vec;
use core::any::TypeId;
use vc_ptr::OwningPtr;
use vc_utils::hash::{SparseHashMap, SparseHashSet};

use super::{ComponentId, ComponentStorage, Components};
use crate::clone::CloneBehavior;
use crate::entity::Entity;
use crate::storage::{Maps, Table, TableRow};
use crate::tick::Tick;
use crate::utils::DebugCheckedUnwrap;

// -----------------------------------------------------------------------------
// Component

pub unsafe trait Component: Sized + Send + Sync + 'static {
    const MUTABLE: bool = true;
    const STORAGE: ComponentStorage = ComponentStorage::Dense;
    const CLONE_BEHAVIOR: CloneBehavior = CloneBehavior::Refuse;


    #[inline(always)]
    #[allow(unused_variables, reason = "default implementation")]
    unsafe fn register_required(registrar: &mut ComponentRegistrar) {}

    #[inline(always)]
    #[allow(unused_variables, reason = "default implementation")]
    unsafe fn collect_required(collector: &mut ComponentCollector) {}

    #[inline(always)]
    #[allow(unused_variables, reason = "default implementation")]
    unsafe fn write_required(writer: &mut ComponentWriter) {}
}

// -----------------------------------------------------------------------------
// ComponentRegistrar

pub struct ComponentRegistrar<'a> {
    pub(crate) components: &'a mut Components,
}

impl<'a> ComponentRegistrar<'a> {
    #[inline]
    pub fn new(components: &'a mut Components) -> Self {
        Self { components }
    }

    #[inline(never)]
    pub fn register<T: Component>(&mut self) {
        self.components.register::<T>();
    }
}

// -----------------------------------------------------------------------------
// ComponentCollector

pub struct ComponentCollector<'a> {
    components: &'a mut Components,
    dense: Vec<ComponentId>,
    sparse: Vec<ComponentId>,
    collected: SparseHashSet<ComponentId>,
}

pub struct CollectResult {
    pub dense: Vec<ComponentId>,
    pub sparse: Vec<ComponentId>,
}

impl<'a> ComponentCollector<'a> {
    #[inline]
    pub fn new(components: &'a mut Components) -> Self {
        ComponentCollector {
            components,
            dense: Vec::new(),
            sparse: Vec::new(),
            collected: SparseHashSet::new(),
        }
    }

    #[inline(never)]
    pub fn collect<T: Component>(&mut self) {
        let id = self.components.register::<T>();
        if self.collected.insert(id) {
            match T::STORAGE {
                ComponentStorage::Dense => {
                    self.dense.push(id);
                }
                ComponentStorage::Sparse => {
                    self.sparse.push(id);
                }
            }
            unsafe {
                T::collect_required(self);
            }
        }
    }

    #[inline]
    pub fn sorted(self) -> CollectResult {
        let mut dense = self.dense;
        let mut sparse = self.sparse;
        dense.sort_unstable();
        sparse.sort_unstable();
        dense.dedup();
        sparse.dedup();
        CollectResult { dense, sparse  }
    }

    #[inline]
    pub fn unsorted(self) -> CollectResult {
        CollectResult { dense: self.dense, sparse: self.sparse  }
    }
}

// -----------------------------------------------------------------------------
// ComponentWriter

pub enum WritedState {
    Required,
    Explicit,
}

pub struct ComponentWriter<'a> {
    pub(crate) data: OwningPtr<'a>,
    pub(crate) components: &'a Components,
    pub(crate) maps: &'a mut Maps,
    pub(crate) table: &'a mut Table,
    pub(crate) entity: Entity,
    pub(crate) table_row: TableRow,
    pub(crate) tick: Tick,
    pub(crate) writed: SparseHashMap<ComponentId, WritedState>,
}

impl ComponentWriter<'_> {
    #[inline(never)]
    pub unsafe fn write_required<T: Component>(&mut self, func: impl FnOnce() -> T) {
        let type_id = TypeId::of::<T>();
        let component = unsafe { self.components.get_id(type_id).debug_checked_unwrap() };
        if !self.writed.contains_key(&component) {
            let data = func();
            vc_ptr::into_owning!(data);
            match T::STORAGE {
                ComponentStorage::Dense => unsafe {
                    self.init_dense(component, data);
                },
                ComponentStorage::Sparse => unsafe {
                    self.init_sparse(component, data);
                },
            }
        }
    }

    #[inline(never)]
    pub unsafe fn write_explicit<T: Component>(&mut self, offset: usize) {
        let type_id = TypeId::of::<T>();
        let component = unsafe { self.components.get_id(type_id).debug_checked_unwrap() };
        match T::STORAGE {
            ComponentStorage::Dense => unsafe {
                self.write_dense(component, offset);
            },
            ComponentStorage::Sparse => unsafe {
                self.write_sparse(component, offset);
            },
        }
    }

    #[inline(never)]
    unsafe fn init_dense(&mut self, component: ComponentId, data: OwningPtr<'_>) {
        unsafe {
            let col = self.table.get_table_col(component).debug_checked_unwrap();
            self.table.init_item(col, self.table_row, data, self.tick);
            self.writed.insert(component, WritedState::Required);
        }
    }

    #[inline(never)]
    unsafe fn init_sparse(&mut self, component: ComponentId, data: OwningPtr<'_>) {
        unsafe {
            let map_id = self.maps.get_id(component).debug_checked_unwrap();
            let map = self.maps.get_unchecked_mut(map_id);
            let row = map.get_map_row(self.entity).debug_checked_unwrap();
            map.init_item(row, data, self.tick);
            self.writed.insert(component, WritedState::Required);
        }
    }

    #[inline(never)]
    unsafe fn write_sparse(&mut self, component: ComponentId, offset: usize) {
        use vc_utils::hash::hash_map::Entry;
        unsafe {
            let data = self.data.borrow_mut().byte_add(offset).promote();
            let map_id = self.maps.get_id(component).debug_checked_unwrap();
            let map = self.maps.get_unchecked_mut(map_id);
            let row = map.get_map_row(self.entity).debug_checked_unwrap();
            match self.writed.entry(component) {
                Entry::Occupied(mut entry) => {
                    map.replace_item(row, data, self.tick);
                    *entry.get_mut() = WritedState::Explicit;
                },
                Entry::Vacant(entry) => {
                    map.init_item(row, data, self.tick);
                    entry.insert(WritedState::Explicit);
                },
            }
        }
    }

    #[inline(never)]
    unsafe fn write_dense(&mut self, component: ComponentId, offset: usize) {
        use vc_utils::hash::hash_map::Entry;
        unsafe {
            let data = self.data.borrow_mut().byte_add(offset).promote();
            let col = self.table.get_table_col(component).debug_checked_unwrap();
            let row = self.table_row;
            match self.writed.entry(component) {
                Entry::Occupied(mut entry) => {
                    self.table.replace_item(col, row, data, self.tick);
                    *entry.get_mut() = WritedState::Explicit;
                },
                Entry::Vacant(entry) => {
                    self.table.init_item(col, row, data, self.tick);
                    entry.insert(WritedState::Explicit);
                },
            }
        }
    }
}
