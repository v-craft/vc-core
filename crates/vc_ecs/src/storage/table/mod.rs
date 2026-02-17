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

use super::{AbortOnPanic, Column, VecRemoveExt};
use crate::borrow::UntypedMut;
use crate::borrow::UntypedRef;
use crate::borrow::UntypedSliceMut;
use crate::borrow::UntypedSliceRef;
use crate::component::ComponentId;
use crate::entity::Entity;
use crate::storage::StorageIndex;
use crate::tick::CheckTicks;
use crate::tick::Tick;
use crate::utils::DebugCheckedUnwrap;

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
            // SAFETY: dense indices stored in self.indices always exist
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
        assert!(self.idents.is_sorted());
        Table {
            columns: self.columns.into_boxed_slice(),
            idents: self.idents.into_boxed_slice(),
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
    entities: Vec<Entity>,
}

impl Debug for Table {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use core::mem::transmute;
        let components = unsafe { transmute::<&[ComponentId], &[u32]>(&self.idents) };
        f.debug_struct("Table")
            .field("components", &components)
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

    pub fn check_ticks(&mut self, check: CheckTicks) {
        let len = self.entity_count();
        self.columns.iter_mut().for_each(|c| unsafe {
            c.check_ticks(len, check);
        });
    }

    /// # Safety
    /// The table must contain this component.
    pub unsafe fn get_index(&self, id: ComponentId) -> StorageIndex {
        let index = unsafe { self.idents.binary_search(&id).debug_checked_unwrap() };
        StorageIndex::new(index as u32)
    }

    pub fn try_get_index(&self, id: ComponentId) -> Option<StorageIndex> {
        if let Ok(index) = self.idents.binary_search(&id) {
            Some(StorageIndex::new(index as u32))
        } else {
            None
        }
    }

    #[inline(always)]
    unsafe fn get_column(&self, index: StorageIndex) -> &Column {
        debug_assert!(index.index() < self.columns.len());

        unsafe { self.columns.get_unchecked(index.index()) }
    }

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
    ) {
        debug_assert!(row.index() < self.entity_count());

        unsafe {
            let column = self.get_column_mut(index);
            column.init_item(row.index(), data, tick);
        }
    }

    /// # Safety:
    /// - `self.is_valid() == true`.
    /// - Run on the correct thread.
    #[inline(always)]
    pub unsafe fn get_ref(
        &self,
        index: StorageIndex,
        row: TableRow,
        last_run: Tick,
        this_run: Tick,
    ) -> UntypedRef<'_> {
        unsafe {
            let col = self.get_column(index);
            col.get_ref(row.index(), last_run, this_run)
        }
    }

    /// # Safety:
    /// - `self.is_valid() == true`.
    /// - Run on the correct thread.
    #[inline(always)]
    pub unsafe fn get_mut(
        &mut self,
        index: StorageIndex,
        row: TableRow,
        last_run: Tick,
        this_run: Tick,
    ) -> UntypedMut<'_> {
        unsafe {
            let col = self.get_column_mut(index);
            col.get_mut(row.index(), last_run, this_run)
        }
    }

    #[inline(always)]
    pub unsafe fn get_slice_ref(
        &self,
        index: StorageIndex,
        last_run: Tick,
        this_run: Tick,
    ) -> UntypedSliceRef<'_> {
        let len = self.entity_count();
        unsafe {
            let col = self.get_column(index);
            col.get_slice_ref(len, last_run, this_run)
        }
    }

    #[inline(always)]
    pub unsafe fn get_slice_mut(
        &mut self,
        index: StorageIndex,
        last_run: Tick,
        this_run: Tick,
    ) -> UntypedSliceMut<'_> {
        let len = self.entity_count();
        unsafe {
            let col = self.get_column_mut(index);
            col.get_slice_mut(len, last_run, this_run)
        }
    }
}

// -----------------------------------------------------------------------------
// Move data

#[derive(Debug)]
pub struct TableRemoveResult {
    pub swapped: Option<Entity>,
}

#[derive(Debug)]
pub struct TableMoveResult {
    pub new_row: TableRow,
    pub swapped: Option<Entity>,
}

impl Table {
    pub unsafe fn swap_remove_and_drop(&mut self, row: TableRow) -> TableRemoveResult {
        let removal_index = row.index();
        let last_index = self.entity_count() - 1;
        debug_assert!(removal_index <= last_index);

        unsafe {
            if removal_index != last_index {
                let swapped = self
                    .entities
                    .copy_return_not_last(last_index, removal_index);

                self.columns.iter_mut().for_each(|c| {
                    c.swap_remove_and_drop_not_last(removal_index, last_index);
                });

                TableRemoveResult {
                    swapped: Some(swapped),
                }
            } else {
                self.entities.set_len(last_index);

                for column in &mut self.columns {
                    column.remove_and_drop_last(last_index);
                }

                TableRemoveResult { swapped: None }
            }
        }
    }

    pub unsafe fn swap_remove_and_forget(&mut self, row: TableRow) -> TableRemoveResult {
        let removal_index = row.index();
        let last_index = self.entity_count() - 1;
        debug_assert!(removal_index <= last_index);

        unsafe {
            if removal_index != last_index {
                let swapped = self
                    .entities
                    .copy_return_not_last(last_index, removal_index);

                self.columns.iter_mut().for_each(|c| {
                    let _ = c.swap_remove_not_last(removal_index, last_index);
                });

                TableRemoveResult {
                    swapped: Some(swapped),
                }
            } else {
                self.entities.set_len(last_index);

                for column in &mut self.columns {
                    let _ = column.remove_last(last_index);
                }

                TableRemoveResult { swapped: None }
            }
        }
    }

    pub unsafe fn move_to_and_forget_missing(
        &mut self,
        row: TableRow,
        other: &mut Table,
    ) -> TableMoveResult {
        let src_index = row.index();
        let last_index = self.entity_count() - 1;
        debug_assert!(src_index <= last_index);

        unsafe {
            if src_index != last_index {
                let moved = self.entities.swap_remove_not_last(src_index, last_index);
                let swapped = *self.entities.get_unchecked(src_index);
                let new_row = other.allocate(moved);
                let dst_index = new_row.index();

                self.idents
                    .iter()
                    .zip(self.columns.iter_mut())
                    .for_each(|(&id, col)| {
                        if let Some(storage_index) = other.try_get_index(id) {
                            let other_col = other.get_column_mut(storage_index);
                            other_col
                                .init_item_from_not_last(col, last_index, src_index, dst_index);
                        } else {
                            let _ = col.swap_remove_not_last(src_index, last_index);
                        }
                    });

                TableMoveResult {
                    new_row,
                    swapped: Some(swapped),
                }
            } else {
                let moved = self.entities.remove_last(last_index);
                let new_row = other.allocate(moved);
                let dst_index = new_row.index();

                self.idents
                    .iter()
                    .zip(self.columns.iter_mut())
                    .for_each(|(&id, col)| {
                        if let Some(storage_index) = other.try_get_index(id) {
                            let other_col = other.get_column_mut(storage_index);
                            other_col.init_item_from_last(col, last_index, dst_index);
                        } else {
                            let _ = col.remove_last(last_index);
                        }
                    });

                TableMoveResult {
                    new_row,
                    swapped: None,
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
        debug_assert!(src_index <= last_index);

        unsafe {
            if src_index != last_index {
                let moved = self.entities.swap_remove_not_last(src_index, last_index);
                let swapped = *self.entities.get_unchecked(src_index);
                let new_row = other.allocate(moved);
                let dst_index = new_row.index();

                self.idents
                    .iter()
                    .zip(self.columns.iter_mut())
                    .for_each(|(&id, col)| {
                        if let Some(storage_index) = other.try_get_index(id) {
                            let other_col = other.get_column_mut(storage_index);
                            other_col
                                .init_item_from_not_last(col, last_index, src_index, dst_index);
                        } else {
                            col.swap_remove_and_drop_not_last(src_index, last_index);
                        }
                    });

                TableMoveResult {
                    new_row,
                    swapped: Some(swapped),
                }
            } else {
                let moved = self.entities.remove_last(last_index);
                let new_row = other.allocate(moved);
                let dst_index = new_row.index();

                self.idents
                    .iter()
                    .zip(self.columns.iter_mut())
                    .for_each(|(&id, col)| {
                        if let Some(storage_index) = other.try_get_index(id) {
                            let other_col = other.get_column_mut(storage_index);
                            other_col.init_item_from_last(col, last_index, dst_index);
                        } else {
                            col.remove_and_drop_last(last_index);
                        }
                    });

                TableMoveResult {
                    new_row,
                    swapped: None,
                }
            }
        }
    }

    pub unsafe fn move_to_and_superset(
        &mut self,
        row: TableRow,
        other: &mut Table,
    ) -> TableMoveResult {
        let src_index = row.index();
        let last_index = self.entity_count() - 1;
        debug_assert!(src_index <= last_index);

        unsafe {
            if src_index != last_index {
                let moved = self.entities.swap_remove_not_last(src_index, last_index);
                let swapped = *self.entities.get_unchecked(src_index);
                let new_row = other.allocate(moved);
                let dst_index = new_row.index();

                self.idents
                    .iter()
                    .zip(self.columns.iter_mut())
                    .for_each(|(&id, col)| {
                        let storage_index = other.get_index(id);
                        let other_col = other.get_column_mut(storage_index);
                        other_col.init_item_from_not_last(col, last_index, src_index, dst_index);
                    });

                TableMoveResult {
                    new_row,
                    swapped: Some(swapped),
                }
            } else {
                let moved = self.entities.remove_last(last_index);
                let new_row = other.allocate(moved);
                let dst_index = new_row.index();

                self.idents
                    .iter()
                    .zip(self.columns.iter_mut())
                    .for_each(|(&id, col)| {
                        let storage_index = other.get_index(id);
                        let other_col = other.get_column_mut(storage_index);
                        other_col.init_item_from_last(col, last_index, dst_index);
                    });

                TableMoveResult {
                    new_row,
                    swapped: None,
                }
            }
        }
    }
}
