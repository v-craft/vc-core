#![allow(clippy::missing_safety_doc, reason = "todo")]

use alloc::vec::Vec;
use core::any::TypeId;
use vc_ptr::OwningPtr;
use vc_utils::extra::TypeIdMap;
use vc_utils::hash::SparseHashSet;

use super::{ComponentId, ComponentStorage, Components};
use crate::clone::CloneBehavior;
use crate::entity::Entity;
use crate::storage::{Maps, Table, TableRow};
use crate::tick::Tick;
use crate::utils::DebugCheckedUnwrap;

// -----------------------------------------------------------------------------
// Component

pub unsafe trait Component: Sized + Send + Sync + 'static {
    const STORAGE: ComponentStorage = ComponentStorage::Dense;
    const MUTABLE: bool = true;
    const CLONE_BEHAVIOR: CloneBehavior = CloneBehavior::Refuse;

    #[inline(always)]
    #[allow(unused_variables, reason = "default")]
    unsafe fn register_required(registrar: &mut ComponentRegistrar) {}

    #[inline(always)]
    #[allow(unused_variables, reason = "default")]
    unsafe fn collect_required(collector: &mut ComponentCollector) {}

    #[inline(always)]
    #[allow(unused_variables, reason = "default")]
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

    #[inline]
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

    #[inline]
    pub fn split(self) -> (Vec<ComponentId>, Vec<ComponentId>) {
        (self.dense, self.sparse)
    }

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
}

// -----------------------------------------------------------------------------
// ComponentWriter

pub enum State {
    Default,
    Custom,
}

pub struct ComponentWriter<'a> {
    pub(crate) data: OwningPtr<'a>,
    pub(crate) components: &'a Components,
    pub(crate) maps: &'a mut Maps,
    pub(crate) table: &'a mut Table,
    pub(crate) entity: Entity,
    pub(crate) table_row: TableRow,
    pub(crate) tick: Tick,
    pub(crate) writed: TypeIdMap<State>,
}

impl ComponentWriter<'_> {
    #[inline]
    pub unsafe fn write_required<T: Component>(&mut self, func: impl FnOnce() -> T) {
        let type_id = TypeId::of::<T>();
        if !self.writed.contains(&type_id) {
            let data = func();
            vc_ptr::into_owning!(data);
            match T::STORAGE {
                ComponentStorage::Dense => unsafe {
                    self.init_dense(type_id, data);
                },
                ComponentStorage::Sparse => unsafe {
                    self.init_sparse(type_id, data);
                },
            }
        }
    }

    #[inline]
    pub unsafe fn write_field<T: Component>(&mut self, offset: usize) {
        let type_id = TypeId::of::<T>();
        match T::STORAGE {
            ComponentStorage::Dense => unsafe {
                self.write_dense(type_id, offset);
            },
            ComponentStorage::Sparse => unsafe {
                self.write_sparse(type_id, offset);
            },
        }
    }

    #[inline(never)]
    unsafe fn init_dense(&mut self, type_id: TypeId, data: OwningPtr<'_>) {
        unsafe {
            let component = self.components.get_id(type_id).debug_checked_unwrap();
            let col = self.table.get_table_col(component).debug_checked_unwrap();
            self.table.init_item(col, self.table_row, data, self.tick);
            self.writed.insert(type_id, State::Default);
        }
    }

    #[inline(never)]
    unsafe fn init_sparse(&mut self, type_id: TypeId, data: OwningPtr<'_>) {
        unsafe {
            let component = self.components.get_id(type_id).debug_checked_unwrap();
            let map_id = self.maps.get_id(component).debug_checked_unwrap();
            let map = self.maps.get_unchecked_mut(map_id);
            let row = map.get_map_row(self.entity).debug_checked_unwrap();
            map.init_item(row, data, self.tick);
            self.writed.insert(type_id, State::Default);
        }
    }

    #[inline(never)]
    unsafe fn write_sparse(&mut self, type_id: TypeId, offset: usize) {
        unsafe {
            let data = self.data.borrow_mut().byte_add(offset).promote();
            let component = self.components.get_id(type_id).debug_checked_unwrap();
            let map_id = self.maps.get_id(component).debug_checked_unwrap();
            let map = self.maps.get_unchecked_mut(map_id);
            let row = map.get_map_row(self.entity).debug_checked_unwrap();
            if self.writed.contains(&type_id) {
                map.replace_item(row, data, self.tick);
            } else {
                map.init_item(row, data, self.tick);
            }
            self.writed.insert(type_id, State::Custom);
        }
    }

    #[inline(never)]
    unsafe fn write_dense(&mut self, type_id: TypeId, offset: usize) {
        unsafe {
            let data = self.data.borrow_mut().byte_add(offset).promote();
            let component = self.components.get_id(type_id).debug_checked_unwrap();
            let col = self.table.get_table_col(component).debug_checked_unwrap();
            if self.writed.contains(&type_id) {
                self.table
                    .replace_item(col, self.table_row, data, self.tick);
            } else {
                self.table.init_item(col, self.table_row, data, self.tick);
            }
            self.writed.insert(type_id, State::Custom);
        }
    }
}
