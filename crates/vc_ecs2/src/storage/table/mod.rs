#![allow(unused_variables, reason = "`DebugLocation` is unused in release mod.")]

// -----------------------------------------------------------------------------
// Module

mod id;
mod tables;

// -----------------------------------------------------------------------------
// Exports

pub use id::{TableId, TableRow};
pub use tables::Tables;

// -----------------------------------------------------------------------------
// Table Implementation

use alloc::boxed::Box;
use alloc::vec::Vec;
use core::alloc::Layout;
use core::fmt::Debug;
use core::num::NonZeroUsize;

use nonmax::NonMaxU32;
use vc_ptr::OwningPtr;
use vc_utils::hash::SparseHashMap;

use super::{AbortOnPanic, Column};
use crate::component::ComponentId;
use crate::entity::Entity;
use crate::storage::StorageIndex;
use crate::tick::CheckTicks;
use crate::tick::Tick;
use crate::utils::{DebugCheckedUnwrap, DebugLocation};

// -----------------------------------------------------------------------------
// TableMoveResult

#[derive(Debug)]
pub struct TableMoveResult {
    pub swapped_entity: Option<Entity>,
    pub new_row: TableRow,
}

// -----------------------------------------------------------------------------
// TableBuilder
pub(super) struct TableBuilder {
    columns: Vec<Column>,
    idents: Vec<ComponentId>,
    indices: SparseHashMap<ComponentId, StorageIndex>,
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
        layout: Layout,
        drop_fn: Option<unsafe fn(OwningPtr<'_>)>,
    ) -> StorageIndex {
        let col = unsafe { Column::new(layout, drop_fn) };

        if let Some(&index) = self.indices.get(&id) {
            // SAFETY: dense indices stored in self.sparse always exist
            unsafe {
                *self.columns.get_unchecked_mut(index.index()) = col;
            }
            index
        } else {
            // SAFETY: `0 < ComponentId < u32::MAX`, so `raw_index < u32::MAX`
            let index = StorageIndex::new(self.columns.len() as u32);

            self.indices.insert(id, index);
            self.columns.push(col);
            self.idents.push(id);

            index
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
    indices: SparseHashMap<ComponentId, StorageIndex>,
    entities: Vec<Entity>,
}

impl Debug for Table {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Table")
            .field("components", &self.idents)
            .field("entities", &self.entities)
            .finish()
    }
}

impl Drop for Table {
    fn drop(&mut self) {
        let len = self.entity_count();
        let current_capacity = self.capacity();
        self.entities.clear();
        self.columns.iter_mut().for_each(|c| unsafe {
            c.dealloc(current_capacity, len);
        });
    }
}

unsafe impl Sync for Table {}
unsafe impl Send for Table {}

// -----------------------------------------------------------------------------
// Basic methods

impl Table {
    #[inline(always)]
    fn capacity(&self) -> usize {
        self.entities.capacity()
    }

    #[inline(always)]
    fn entity_count(&self) -> usize {
        self.entities.len()
    }

    pub fn check_ticks(&mut self, check: CheckTicks) {
        let len = self.entity_count();
        self.columns.iter_mut().for_each(|c| unsafe {
            c.check_ticks(len, check);
        });
    }

    /// # Safety
    /// The table must contain this component.
    unsafe fn get_index(&self, id: ComponentId) -> StorageIndex {
        unsafe { *self.indices.get(&id).debug_checked_unwrap() }
    }

    // #[inline(always)]
    // unsafe fn get_column(&self, index: StorageIndex) -> &Column {
    //     debug_assert!(index.index() < self.columns.len());

    //     unsafe { self.columns.get_unchecked(index.index()) }
    // }

    #[inline(always)]
    unsafe fn get_column_mut(&mut self, index: StorageIndex) -> &mut Column {
        debug_assert!(index.index() < self.columns.len());

        unsafe { self.columns.get_unchecked_mut(index.index()) }
    }

    #[inline]
    pub unsafe fn init_component(
        &mut self,
        index: StorageIndex,
        row: TableRow,
        data: OwningPtr<'_>,
        tick: Tick,
        caller: DebugLocation,
    ) {
        debug_assert!(row.index() < self.entity_count());

        unsafe {
            let column = self.get_column_mut(index);
            column.init_item(row.index(), data, tick, caller);
        }
    }

    pub unsafe fn allocate(&mut self, entity: Entity) -> TableRow {
        #[cold]
        #[inline(never)]
        fn reserve_one(this: &mut Table) {
            let abort_guard = AbortOnPanic;

            let old_capacity = this.entities.capacity();
            this.entities.reserve(1);
            let new_capacity = this.entities.capacity();

            unsafe {
                let new_capacity = NonZeroUsize::new_unchecked(new_capacity);
                if let Some(current) = NonZeroUsize::new(old_capacity) {
                    this.columns.iter_mut().for_each(|c| {
                        c.realloc(current, new_capacity);
                    });
                } else {
                    this.columns.iter_mut().for_each(|c| c.alloc(new_capacity));
                }
            }

            ::core::mem::forget(abort_guard);
        }

        let len = self.entities.len();
        if len == self.entities.capacity() {
            reserve_one(self);
        }

        self.entities.push(entity);

        // SAFETY: `0 < EntityId < u32::MAX`, so `len < u32::MAX`
        unsafe { TableRow::new(NonMaxU32::new_unchecked(len as u32)) }
    }
}

// -----------------------------------------------------------------------------
// Optional methods

impl Table {
    //     #[inline]
    //     pub unsafe fn get_ptr(&self, raw_index: u32, row: TableRow) -> Ptr<'_> {
    //         cfg::debug! { assert!(row.index() < self.entity_count()); }
    //         unsafe { self.get_column(raw_index).get_data_ptr(row.index()) }
    //     }

    //     #[inline]
    //     pub unsafe fn get_ptr_mut(&mut self, raw_index: u32, row: TableRow) -> PtrMut<'_> {
    //         cfg::debug! { assert!(row.index() < self.entity_count()); }
    //         unsafe { self.get_column_mut(raw_index).get_data_ptr_mut(row.index()) }
    //     }

    //     #[inline]
    //     pub unsafe fn take(&mut self, raw_index: u32, row: TableRow) -> OwningPtr<'_> {
    //         cfg::debug! { assert!(row.index() < self.entity_count()); }
    //         unsafe {
    //             self.get_column_mut(raw_index)
    //                 .get_data_ptr_mut(row.index())
    //                 .promote()
    //         }
    //     }

    //     #[inline]
    //     pub unsafe fn get_ticks(
    //         &self,
    //         raw_index: u32,
    //         row: TableRow,
    //     ) -> ComponentTicks {
    //         unsafe {
    //             let row_index = row.index();
    //             let col = self.get_column(raw_index);
    //             ComponentTicks {
    //                 added: col.get_added_tick(row_index),
    //                 changed: col.get_changed_tick(row_index),
    //             }
    //         }
    //     }

    //     #[inline]
    //     pub unsafe fn get_changed_by(
    //         &self,
    //         raw_index: u32,
    //         row: TableRow,
    //     ) -> DebugLocation {
    //         unsafe {
    //             let row_index = row.index();
    //             let col = self.get_column(raw_index);
    //             col.get_changed_by(row_index)
    //         }
    //     }

    //     /// # Safety:
    //     /// - `self.is_valid() == true`.
    //     /// - Run on the correct thread.
    //     #[inline(always)]
    //     pub unsafe fn get_untyped(
    //         &self,
    //         raw_index: u32,
    //         row: TableRow,
    //         last_run: Tick,
    //         this_run: Tick,
    //     ) -> UntypedRef<'_> {
    //         unsafe {
    //             let row_index = row.index();
    //             let col = self.get_column(raw_index);

    //             UntypedRef {
    //                 value: col.get_data_ptr(row_index),
    //                 ticks: ComponentTicksRef {
    //                     added: &*col.get_added_tick_cell(row_index).get(),
    //                     changed: &*col.get_changed_tick_cell(row_index).get(),
    //                     changed_by: col.get_changed_by_cell(row_index).map(|cb| &*cb.get()),
    //                     last_run,
    //                     this_run,
    //                 },
    //             }
    //         }
    //     }

    //     /// # Safety:
    //     /// - `self.is_valid() == true`.
    //     /// - Run on the correct thread.
    //     #[inline(always)]
    //     pub unsafe fn get_untyped_mut(
    //         &mut self,
    //         raw_index: u32,
    //         row: TableRow,
    //         last_run: Tick,
    //         this_run: Tick,
    //     ) -> UntypedMut<'_> {
    //         unsafe {
    //             let row_index = row.index();
    //             let col = self.get_column_mut(raw_index);

    //             UntypedMut {
    //                 ticks: ComponentTicksMut {
    //                     added: &mut *col.get_added_tick_cell(row_index).get(),
    //                     changed: &mut *col.get_changed_tick_cell(row_index).get(),
    //                     changed_by: col.get_changed_by_cell(row_index).map(|cb| &mut *cb.get()),
    //                     last_run,
    //                     this_run,
    //                 },
    //                 value: col.get_data_ptr_mut(row_index),
    //             }
    //         }
    //     }

    //     #[inline]
    //     pub unsafe fn update(
    //         &mut self,
    //         raw_index: u32,
    //         row: TableRow,
    //         data: OwningPtr<'_>,
    //         tick: Tick,
    //         caller: DebugLocation,
    //     ) {
    //         cfg::debug! { assert!(row.index() < self.entity_count()); }
    //         unsafe {
    //             let column = self.get_column_mut(raw_index);
    //             column.replace_item(row.index(), data, tick, caller);
    //         }
    //     }

    //     #[inline]
    //     pub unsafe fn get_drop_fn(&self, raw_index: u32) -> Option<unsafe fn(OwningPtr<'_>)> {
    //         unsafe { self.get_column(raw_index).drop_fn() }
    //     }

    //     #[inline]
    //     pub unsafe fn get_data_cell_slice<T>(&self, raw_index: u32) -> &[UnsafeCell<T>] {
    //         unsafe {
    //             self.get_column(raw_index)
    //                 .get_data_cell_slice(self.entity_count())
    //         }
    //     }

    //     #[inline]
    //     pub unsafe fn get_added_ticks_cell_slice(&self, raw_index: u32) -> &[UnsafeCell<Tick>] {
    //         unsafe {
    //             self.get_column(raw_index)
    //                 .get_added_ticks_cell_slice(self.entity_count())
    //         }
    //     }

    //     #[inline]
    //     pub unsafe fn get_changed_ticks_cell_slice(&self, raw_index: u32) -> &[UnsafeCell<Tick>] {
    //         unsafe {
    //             self.get_column(raw_index)
    //                 .get_changed_ticks_cell_slice(self.entity_count())
    //         }
    //     }

    //     #[inline]
    //     pub unsafe fn get_changed_by_cell_slice(
    //         &self,
    //         raw_index: u32,
    //     ) -> DebugLocation<&[UnsafeCell<&'static Location<'static>>]> {
    //         unsafe {
    //             self.get_column(raw_index)
    //                 .get_changed_by_cell_slice(self.entity_count())
    //         }
    //     }

    //     #[inline]
    //     pub unsafe fn get_added_tick_cell(
    //         &self,
    //         raw_index: u32,
    //         row: TableRow,
    //     ) -> &UnsafeCell<Tick> {
    //         unsafe { self.get_column(raw_index).get_added_tick_cell(row.index()) }
    //     }

    //     #[inline]
    //     pub unsafe fn get_changed_tick_cell(
    //         &self,
    //         raw_index: u32,
    //         row: TableRow,
    //     ) -> &UnsafeCell<Tick> {
    //         unsafe { self.get_column(raw_index).get_changed_tick_cell(row.index()) }
    //     }

    //     #[inline]
    //     pub unsafe fn get_changed_by_cell(
    //         &self,
    //         raw_index: u32,
    //         row: TableRow,
    //     ) -> DebugLocation<&UnsafeCell<&'static Location<'static>>> {
    //         unsafe { self.get_column(raw_index).get_changed_by_cell(row.index()) }
    //     }

    //     pub unsafe fn swap_remove(&mut self, row: TableRow) -> Option<Entity> {
    //         use crate::storage::VecCopyRemove;

    //         let removal_index = row.index();
    //         let last_index = self.entity_count() - 1;

    //         cfg::debug! { assert!(removal_index <= last_index); }

    //         unsafe {
    //             if removal_index != last_index {
    //                 let entity = self
    //                     .entities
    //                     .copy_remove_last(removal_index, last_index);

    //                 for column in &mut self.columns {
    //                     column.swap_remove_and_drop_nonoverlapping(removal_index, last_index);
    //                 }
    //                 Some(entity)
    //             } else {
    //                 self.entities.set_len(last_index);

    //                 for column in &mut self.columns {
    //                     column.drop_last(last_index);
    //                 }

    //                 None
    //             }
    //         }
    //     }

    //     pub unsafe fn move_to_and_forget_missing(
    //         &mut self,
    //         row: TableRow,
    //         other: &mut Table,
    //     ) -> TableMoveResult {
    //         let src_index = row.index();
    //         let last_index = self.entity_count() - 1;

    //         cfg::debug! { assert!(src_index < self.entity_count()); }

    //         if src_index != last_index {
    //             unsafe {
    //                 let dst_row = other.allocate(
    //                     self.entities
    //                         .swap_remove_nonoverlapping(src_index, last_index),
    //                 );
    //                 let dst_index = dst_row.index();

    //                 for (id, column) in self.idents.iter().zip(self.columns.iter_mut()) {
    //                     if let Some(raw_index) = other.get_raw_index(*id) {
    //                         let other_col = other.get_column_mut(raw_index);
    //                         other_col.init_item_from_nonoverlapping(
    //                             column, last_index, src_index, dst_index,
    //                         );
    //                     } else {
    //                         let _ = column.swap_remove_nonoverlapping(src_index, last_index);
    //                     }
    //                 }

    //                 TableMoveResult {
    //                     new_row: dst_row,
    //                     swapped_entity: Some(*self.entities.get_unchecked(src_index)),
    //                 }
    //             }
    //         } else {
    //             unsafe {
    //                 let dst_row = other.allocate(self.entities.remove_last(last_index));
    //                 let dst_index = dst_row.index();

    //                 for (id, column) in self.idents.iter().zip(self.columns.iter_mut()) {
    //                     if let Some(raw_index) = other.get_raw_index(*id) {
    //                         let other_col = other.get_column_mut(raw_index);
    //                         other_col.init_item_from_last(column, last_index, dst_index);
    //                     } else {
    //                         let _ = column.remove_last(last_index);
    //                     }
    //                 }

    //                 TableMoveResult {
    //                     new_row: dst_row,
    //                     swapped_entity: None,
    //                 }
    //             }
    //         }
    //     }

    //     pub unsafe fn move_to_and_drop_missing(
    //         &mut self,
    //         row: TableRow,
    //         other: &mut Table,
    //     ) -> TableMoveResult {
    //         let src_index = row.index();
    //         let last_index = self.entity_count() - 1;

    //         cfg::debug! { assert!(src_index < self.entity_count()); }

    //         if src_index != last_index {
    //             unsafe {
    //                 let dst_row = other.allocate(
    //                     self.entities
    //                         .swap_remove_nonoverlapping(src_index, last_index),
    //                 );
    //                 let dst_index = dst_row.index();

    //                 for (id, column) in self.idents.iter().zip(self.columns.iter_mut()) {
    //                     if let Some(raw_index) = other.get_raw_index(*id) {
    //                         let other_col = other.get_column_mut(raw_index);
    //                         other_col.init_item_from_nonoverlapping(
    //                             column, last_index, src_index, dst_index,
    //                         );
    //                     } else {
    //                         column.swap_remove_and_drop_nonoverlapping(src_index, last_index);
    //                     }
    //                 }

    //                 TableMoveResult {
    //                     new_row: dst_row,
    //                     swapped_entity: Some(*self.entities.get_unchecked(src_index)),
    //                 }
    //             }
    //         } else {
    //             unsafe {
    //                 let dst_row = other.allocate(self.entities.remove_last(last_index));
    //                 let dst_index = dst_row.index();

    //                 for (id, column) in self.idents.iter().zip(self.columns.iter_mut()) {
    //                     if let Some(raw_index) = other.get_raw_index(*id) {
    //                         let other_col = other.get_column_mut(raw_index);
    //                         other_col.init_item_from_last(column, last_index, dst_index);
    //                     } else {
    //                         column.drop_last(last_index);
    //                     }
    //                 }

    //                 TableMoveResult {
    //                     new_row: dst_row,
    //                     swapped_entity: None,
    //                 }
    //             }
    //         }
    //     }

    //     pub unsafe fn move_to_superset(&mut self, row: TableRow, other: &mut Table) -> TableMoveResult {
    //         let src_index = row.index();
    //         let last_index = self.entity_count() - 1;

    //         cfg::debug! { assert!(src_index < self.entity_count()); }

    //         if src_index != last_index {
    //             unsafe {
    //                 let dst_row = other.allocate(
    //                     self.entities
    //                         .swap_remove_nonoverlapping(src_index, last_index),
    //                 );
    //                 let dst_index = dst_row.index();

    //                 for (id, column) in self.idents.iter().zip(self.columns.iter_mut()) {
    //                     let raw_index = other.get_raw_index(*id).debug_checked_unwrap();
    //                     let other_col = other.get_column_mut(raw_index);
    //                     other_col
    //                         .init_item_from_nonoverlapping(column, last_index, src_index, dst_index);
    //                 }

    //                 TableMoveResult {
    //                     new_row: dst_row,
    //                     swapped_entity: Some(*self.entities.get_unchecked(src_index)),
    //                 }
    //             }
    //         } else {
    //             unsafe {
    //                 let dst_row = other.allocate(self.entities.remove_last(last_index));
    //                 let dst_index = dst_row.index();

    //                 for (id, column) in self.idents.iter().zip(self.columns.iter_mut()) {
    //                     let raw_index = other.get_raw_index(*id).debug_checked_unwrap();
    //                     let other_col = other.get_column_mut(raw_index);
    //                     other_col.init_item_from_last(column, last_index, dst_index);
    //                 }

    //                 TableMoveResult {
    //                     new_row: dst_row,
    //                     swapped_entity: None,
    //                 }
    //             }
    //         }
    //     }
}
