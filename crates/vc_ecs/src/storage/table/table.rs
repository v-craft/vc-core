#![expect(unsafe_code, reason = "original implementation requires unsafe code.")]

use alloc::boxed::Box;
use alloc::vec::Vec;
use core::alloc::Layout;
use core::cell::UnsafeCell;
use core::num::NonZeroUsize;
use core::panic::Location;

use nonmax::NonMaxU32;
use vc_ptr::{OwningPtr, Ptr, PtrMut};
use vc_utils::hash::SparseHashMap;

use super::TableRow;
use crate::cfg;
use crate::component::{ComponentId, ComponentTicks};
use crate::entity::Entity;
use crate::storage::{AbortOnPanic, Column, VecSwapRemove};
use crate::tick::CheckTicks;
use crate::tick::Tick;
use crate::utils::{DebugCheckedUnwrap, DebugLocation};

// -----------------------------------------------------------------------------
// TableMoveResult

pub struct TableMoveResult {
    pub swapped_entity: Option<Entity>,
    pub new_row: TableRow,
}

// -----------------------------------------------------------------------------
// TableBuilder
pub struct TableBuilder {
    columns: Vec<Column>,
    idents: Vec<ComponentId>,
    indices: SparseHashMap<ComponentId, u32>,
}

impl TableBuilder {
    pub fn new(column_count: usize) -> Self {
        let hash_capacity = column_count + (column_count >> 1);

        Self {
            columns: Vec::with_capacity(column_count),
            idents: Vec::with_capacity(column_count),
            indices: SparseHashMap::with_capacity(hash_capacity),
        }
    }

    pub unsafe fn insert(
        &mut self,
        id: ComponentId,
        item_layout: Layout,
        drop_fn: Option<unsafe fn(OwningPtr<'_>)>,
    ) -> u32 {
        let col = unsafe { Column::empty(item_layout, drop_fn) };

        if let Some(&raw_index) = self.indices.get(&id) {
            // SAFETY: dense indices stored in self.sparse always exist
            unsafe {
                *self.columns.get_unchecked_mut(raw_index as usize) = col;
            }
            raw_index
        } else {
            // SAFETY: `0 < ComponentId < u32::MAX`, so `raw_index < u32::MAX`
            let raw_index = self.columns.len() as u32;

            self.indices.insert(id, raw_index);
            self.columns.push(col);
            self.idents.push(id);

            raw_index
        }
    }

    #[must_use]
    #[inline]
    pub fn build(self) -> Table {
        Table {
            columns: self.columns.into_boxed_slice(),
            idents: self.idents.into_boxed_slice(),
            indices: self.indices,
            // SAFETY: `capacity` must be `0`, because columns is unallocated.
            entities: Vec::new(),
        }
    }
}

// -----------------------------------------------------------------------------
// Table

pub struct Table {
    columns: Box<[Column]>,
    idents: Box<[ComponentId]>,
    indices: SparseHashMap<ComponentId, u32>,
    entities: Vec<Entity>,
}

impl Drop for Table {
    fn drop(&mut self) {
        let len = self.entity_count();
        let current_capacity = self.capacity();
        self.entities.clear();
        for column in &mut self.columns {
            unsafe {
                column.dealloc(current_capacity, len);
            }
        }
    }
}

impl Table {
    #[inline(always)]
    pub fn capacity(&self) -> usize {
        self.entities.capacity()
    }

    #[inline(always)]
    pub fn component_count(&self) -> usize {
        self.columns.len()
    }

    #[inline(always)]
    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }

    #[inline(always)]
    pub fn entities(&self) -> &[Entity] {
        &self.entities
    }

    pub fn clear_entities(&mut self) {
        let len = self.entity_count();
        self.entities.clear();
        for column in &mut self.columns {
            unsafe {
                column.clear(len);
            }
        }
    }

    pub fn dealloc(&mut self) {
        let len = self.entity_count();
        let current_capacity = self.capacity();

        self.entities = Vec::new();

        for column in &mut self.columns {
            unsafe {
                column.dealloc(current_capacity, len);
            }
        }
    }

    pub fn check_ticks(&mut self, check: CheckTicks) {
        let len = self.entity_count();
        for column in &mut self.columns {
            unsafe {
                column.check_ticks(len, check);
            }
        }
    }

    #[inline]
    pub fn contains_component(&self, id: ComponentId) -> bool {
        self.indices.contains_key(&id)
    }

    #[inline]
    pub fn get_raw_index(&self, id: ComponentId) -> Option<u32> {
        self.indices.get(&id).copied()
    }

    #[inline(always)]
    unsafe fn get_column(&self, raw_index: u32) -> &Column {
        cfg::debug! { assert!((raw_index as usize) < self.columns.len()); }
        unsafe { self.columns.get_unchecked(raw_index as usize) }
    }

    #[inline(always)]
    unsafe fn get_column_mut(&mut self, raw_index: u32) -> &mut Column {
        cfg::debug! { assert!((raw_index as usize) < self.columns.len()); }
        unsafe { self.columns.get_unchecked_mut(raw_index as usize) }
    }

    #[inline(always)]
    pub unsafe fn get_component(&self, raw_index: u32, row: TableRow) -> Ptr<'_> {
        cfg::debug! { assert!(row.index() < self.entity_count()); }
        unsafe { self.get_column(raw_index).get_data(row.index()) }
    }

    #[inline(always)]
    pub unsafe fn get_component_mut(&mut self, raw_index: u32, row: TableRow) -> PtrMut<'_> {
        cfg::debug! { assert!(row.index() < self.entity_count()); }
        unsafe { self.get_column_mut(raw_index).get_data_mut(row.index()) }
    }

    #[inline(always)]
    pub unsafe fn take_component(&mut self, raw_index: u32, row: TableRow) -> OwningPtr<'_> {
        cfg::debug! { assert!(row.index() < self.entity_count()); }
        unsafe {
            self.get_column_mut(raw_index)
                .get_data_mut(row.index())
                .promote()
        }
    }

    #[inline(always)]
    pub unsafe fn init_component(
        &mut self,
        raw_index: u32,
        row: TableRow,
        data: OwningPtr<'_>,
        tick: Tick,
        caller: DebugLocation,
    ) {
        cfg::debug! { assert!(row.index() < self.entity_count()); }
        unsafe {
            let column = self.get_column_mut(raw_index);
            column.init_item(row.index(), data, tick, caller);
        }
    }

    #[inline]
    pub unsafe fn get_drop_fn(&self, raw_index: u32) -> Option<unsafe fn(OwningPtr<'_>)> {
        unsafe { self.get_column(raw_index).drop_fn() }
    }

    #[inline]
    pub unsafe fn get_data_slice<T>(&self, raw_index: u32) -> &[UnsafeCell<T>] {
        unsafe {
            self.get_column(raw_index)
                .get_data_slice(self.entity_count())
        }
    }

    #[inline]
    pub unsafe fn get_added_ticks_slice(&self, raw_index: u32) -> &[UnsafeCell<Tick>] {
        unsafe {
            self.get_column(raw_index)
                .get_added_ticks_slice(self.entity_count())
        }
    }

    #[inline]
    pub unsafe fn get_changed_ticks_slice(&self, raw_index: u32) -> &[UnsafeCell<Tick>] {
        unsafe {
            self.get_column(raw_index)
                .get_changed_ticks_slice(self.entity_count())
        }
    }

    #[inline]
    pub unsafe fn get_changed_by_slice(
        &self,
        raw_index: u32,
    ) -> DebugLocation<&[UnsafeCell<&'static Location<'static>>]> {
        unsafe {
            self.get_column(raw_index)
                .get_changed_by_slice(self.entity_count())
        }
    }

    #[inline]
    pub unsafe fn get_changed_tick(
        &self,
        raw_index: u32,
        row: TableRow,
    ) -> Option<&UnsafeCell<Tick>> {
        let index = row.index();
        if index < self.entity_count() {
            let ret = unsafe { self.get_column(raw_index).get_changed_tick(index) };
            Some(ret)
        } else {
            None
        }
    }

    #[inline]
    pub unsafe fn get_added_tick(
        &self,
        raw_index: u32,
        row: TableRow,
    ) -> Option<&UnsafeCell<Tick>> {
        let index = row.index();
        if index < self.entity_count() {
            let ret = unsafe { self.get_column(raw_index).get_added_tick(index) };
            Some(ret)
        } else {
            None
        }
    }

    #[inline]
    pub unsafe fn get_changed_by(
        &self,
        _raw_index: u32,
        _row: TableRow,
    ) -> DebugLocation<Option<&UnsafeCell<&'static Location<'static>>>> {
        cfg::debug! {
            let index = _row.index();
            if index < self.entity_count() {
                let ret = unsafe {
                    self.get_column(_raw_index).get_changed_by(index)
                };
                return  DebugLocation::untranspose(Some(ret));
            }
        }

        DebugLocation::new(None)
    }

    #[inline]
    pub unsafe fn get_component_ticks(
        &self,
        raw_index: u32,
        row: TableRow,
    ) -> Option<ComponentTicks> {
        let index = row.index();
        if index < self.entity_count() {
            unsafe {
                let col = self.get_column(raw_index);
                Some(ComponentTicks {
                    added: col.copy_added_tick(index),
                    changed: col.copy_changed_tick(index),
                })
            }
        } else {
            None
        }
    }

    pub unsafe fn swap_remove(&mut self, row: TableRow) -> Option<Entity> {
        use crate::storage::VecCopyRemove;

        let removal_index = row.index();
        let last_index = self.entity_count() - 1;

        cfg::debug! { assert!(removal_index <= last_index); }

        unsafe {
            if removal_index != last_index {
                let entity = self
                    .entities
                    .copy_last_and_return_nonoverlapping(removal_index, last_index);

                for column in &mut self.columns {
                    column.swap_remove_and_drop_nonoverlapping(removal_index, last_index);
                }
                Some(entity)
            } else {
                self.entities.set_len(last_index);
                for column in &mut self.columns {
                    column.drop_last(last_index);
                }
                None
            }
        }
    }

    #[inline]
    unsafe fn alloc_columns(&mut self, new_capacity: NonZeroUsize) {
        for col in &mut self.columns {
            unsafe {
                col.alloc(new_capacity);
            }
        }
    }

    #[inline]
    unsafe fn realloc_columns(
        &mut self,
        current_capacity: NonZeroUsize,
        new_capacity: NonZeroUsize,
    ) {
        for col in &mut self.columns {
            unsafe {
                col.realloc(current_capacity, new_capacity);
            }
        }
    }

    #[cold]
    #[inline(never)]
    fn reserve_one(&mut self) {
        let old_capacity = self.capacity();

        let abort_guard = AbortOnPanic;

        self.entities.reserve(1);

        let new_capacity = self.entities.capacity();

        unsafe {
            let new_capacity = NonZeroUsize::new_unchecked(new_capacity);

            if let Some(current_capacity) = NonZeroUsize::new(old_capacity) {
                self.realloc_columns(current_capacity, new_capacity);
            } else {
                self.alloc_columns(new_capacity);
            }
        }

        ::core::mem::forget(abort_guard);
    }

    pub unsafe fn allocate(&mut self, entity: Entity) -> TableRow {
        let len = self.entities.len();

        if len == self.entities.capacity() {
            self.reserve_one();
        }

        self.entities.push(entity);

        for col in &mut self.columns {
            unsafe {
                col.reset_item(len);
            }
        }

        // SAFETY: `0 < EntityId < u32::MAX`, so `len < u32::MAX`
        unsafe { TableRow::new(NonMaxU32::new_unchecked(len as u32)) }
    }

    pub unsafe fn move_to_and_forget_missing(
        &mut self,
        row: TableRow,
        other: &mut Table,
    ) -> TableMoveResult {
        let src_index = row.index();
        let last_index = self.entity_count() - 1;

        cfg::debug! { assert!(src_index < self.entity_count()); }

        if src_index != last_index {
            unsafe {
                let dst_row = other.allocate(
                    self.entities
                        .swap_remove_nonoverlapping(src_index, last_index),
                );
                let dst_index = dst_row.index();

                for (id, column) in self.idents.iter().zip(self.columns.iter_mut()) {
                    if let Some(raw_index) = other.get_raw_index(*id) {
                        let other_col = other.get_column_mut(raw_index);
                        other_col.init_item_from_nonoverlapping(
                            column, last_index, src_index, dst_index,
                        );
                    } else {
                        let _ = column.swap_remove_nonoverlapping(src_index, last_index);
                    }
                }

                TableMoveResult {
                    new_row: dst_row,
                    swapped_entity: Some(*self.entities.get_unchecked(src_index)),
                }
            }
        } else {
            unsafe {
                let dst_row = other.allocate(self.entities.remove_last(last_index));
                let dst_index = dst_row.index();

                for (id, column) in self.idents.iter().zip(self.columns.iter_mut()) {
                    if let Some(raw_index) = other.get_raw_index(*id) {
                        let other_col = other.get_column_mut(raw_index);
                        other_col.init_item_from_last(column, last_index, dst_index);
                    } else {
                        let _ = column.remove_last(last_index);
                    }
                }

                TableMoveResult {
                    new_row: dst_row,
                    swapped_entity: None,
                }
            }
        }
    }

    pub unsafe fn move_to_and_drop_missing(
        &mut self,
        row: TableRow,
        other: &mut Table,
    ) -> TableMoveResult {
        let src_index = row.index();
        let last_index = self.entity_count() - 1;

        cfg::debug! { assert!(src_index < self.entity_count()); }

        if src_index != last_index {
            unsafe {
                let dst_row = other.allocate(
                    self.entities
                        .swap_remove_nonoverlapping(src_index, last_index),
                );
                let dst_index = dst_row.index();

                for (id, column) in self.idents.iter().zip(self.columns.iter_mut()) {
                    if let Some(raw_index) = other.get_raw_index(*id) {
                        let other_col = other.get_column_mut(raw_index);
                        other_col.init_item_from_nonoverlapping(
                            column, last_index, src_index, dst_index,
                        );
                    } else {
                        column.swap_remove_and_drop_nonoverlapping(src_index, last_index);
                    }
                }

                TableMoveResult {
                    new_row: dst_row,
                    swapped_entity: Some(*self.entities.get_unchecked(src_index)),
                }
            }
        } else {
            unsafe {
                let dst_row = other.allocate(self.entities.remove_last(last_index));
                let dst_index = dst_row.index();

                for (id, column) in self.idents.iter().zip(self.columns.iter_mut()) {
                    if let Some(raw_index) = other.get_raw_index(*id) {
                        let other_col = other.get_column_mut(raw_index);
                        other_col.init_item_from_last(column, last_index, dst_index);
                    } else {
                        column.drop_last(last_index);
                    }
                }

                TableMoveResult {
                    new_row: dst_row,
                    swapped_entity: None,
                }
            }
        }
    }

    pub unsafe fn move_to_superset(&mut self, row: TableRow, other: &mut Table) -> TableMoveResult {
        let src_index = row.index();
        let last_index = self.entity_count() - 1;

        cfg::debug! { assert!(src_index < self.entity_count()); }

        if src_index != last_index {
            unsafe {
                let dst_row = other.allocate(
                    self.entities
                        .swap_remove_nonoverlapping(src_index, last_index),
                );
                let dst_index = dst_row.index();

                for (id, column) in self.idents.iter().zip(self.columns.iter_mut()) {
                    let raw_index = other.get_raw_index(*id).debug_checked_unwrap();
                    let other_col = other.get_column_mut(raw_index);
                    other_col
                        .init_item_from_nonoverlapping(column, last_index, src_index, dst_index);
                }

                TableMoveResult {
                    new_row: dst_row,
                    swapped_entity: Some(*self.entities.get_unchecked(src_index)),
                }
            }
        } else {
            unsafe {
                let dst_row = other.allocate(self.entities.remove_last(last_index));
                let dst_index = dst_row.index();

                for (id, column) in self.idents.iter().zip(self.columns.iter_mut()) {
                    let raw_index = other.get_raw_index(*id).debug_checked_unwrap();
                    let other_col = other.get_column_mut(raw_index);
                    other_col.init_item_from_last(column, last_index, dst_index);
                }

                TableMoveResult {
                    new_row: dst_row,
                    swapped_entity: None,
                }
            }
        }
    }
}
