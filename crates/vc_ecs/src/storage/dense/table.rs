use alloc::boxed::Box;
use alloc::vec::Vec;
use core::alloc::Layout;
use core::fmt::Debug;
use core::num::NonZeroUsize;
use core::slice;

use vc_ptr::OwningPtr;
use vc_ptr::Ptr;

use super::{TableCol, TableRow};
use crate::borrow::UntypedMut;
use crate::borrow::UntypedRef;
use crate::borrow::UntypedSliceMut;
use crate::borrow::UntypedSliceRef;
use crate::component::ComponentId;
use crate::entity::Entity;
use crate::entity::MovedEntity;
use crate::storage::{AbortOnPanic, Column, VecRemoveExt};
use crate::tick::CheckTicks;
use crate::tick::Tick;
use crate::utils::Dropper;

// -----------------------------------------------------------------------------
// TableBuilder

/// Builder for creating a new `Table` with a fixed set of component columns.
pub(super) struct TableBuilder {
    columns: Vec<Column>,
    idents: Vec<ComponentId>,
}

impl TableBuilder {
    /// Creates a new builder with pre-allocated capacity for the given column count.
    pub fn new(column_count: usize) -> Self {
        Self {
            columns: Vec::with_capacity(column_count),
            idents: Vec::with_capacity(column_count),
        }
    }

    /// Inserts a new component column into the table being built.
    ///
    /// # Safety
    /// - Inserted `ComponentId`s must be unique and sorted in ascending order
    /// - `layout` and `drop_fn` must correctly match the component type
    /// - The column index returned is valid for the lifetime of the built table
    pub unsafe fn insert(
        &mut self,
        id: ComponentId,
        layout: Layout,
        dropper: Option<Dropper>,
    ) -> TableCol {
        // `0 < ComponentId < u32::MAX`, so `location < u32::MAX`
        let index = self.columns.len() as u32;
        self.columns.push(unsafe { Column::new(layout, dropper) });
        self.idents.push(id);

        TableCol(index)
    }

    /// Consumes the builder and creates the final `Table`.
    ///
    /// # Panics
    /// Panics if component IDs are not unique or not properly sorted.
    #[must_use]
    pub fn build(mut self) -> Table {
        assert!(self.idents.is_sorted());
        let len = self.idents.len();
        self.idents.dedup();
        assert!(self.idents.len() == len);

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

/// A dense columnar storage table for ECS components.
///
/// Organizes components by type (columns) and entities (rows), providing
/// efficient batch access and cache-friendly iteration.
pub struct Table {
    columns: Box<[Column]>,
    idents: Box<[ComponentId]>,
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
        self.columns.iter_mut().for_each(|c| unsafe {
            c.drop_slice(len);
            c.dealloc(current_capacity);
        });
    }
}

unsafe impl Sync for Table {}
unsafe impl Send for Table {}

impl Table {
    /// Returns the current allocation capacity of the table.
    #[inline(always)]
    fn capacity(&self) -> usize {
        self.entities.capacity()
    }

    /// Returns the number of entities currently stored in the table.
    #[inline(always)]
    fn entity_count(&self) -> usize {
        self.entities.len()
    }

    #[inline(always)]
    pub fn entities(&self) -> &[Entity] {
        &self.entities
    }

    /// Allocates space for a new entity and returns its row index.
    ///
    /// # Safety
    /// - The entity must be unique within this table
    /// - The returned row is valid until the entity is removed
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
                    this.columns.iter_mut().for_each(|col| {
                        col.realloc(current, new_capacity);
                    });
                } else {
                    this.columns
                        .iter_mut()
                        .for_each(|col| col.alloc(new_capacity));
                }
            }

            ::core::mem::forget(abort_guard);
        }

        let len = self.entities.len();
        if len == self.entities.capacity() {
            reserve_one(self);
        }

        self.entities.push(entity);
        // `0 < EntityId < u32::MAX`, so `len < u32::MAX`
        TableRow(len as u32)
    }

    /// Finds the column index for a given component ID using binary search.
    ///
    /// # Complexity
    /// O(log n) where n is the number of component types
    pub fn get_table_col(&self, key: ComponentId) -> Option<TableCol> {
        let index = self.idents.binary_search(&key).ok()?;
        Some(TableCol(index as u32))
    }

    /// Finds the row index for a given entity using linear search.
    ///
    /// # Complexity
    /// O(n) where n is the number of entities
    ///
    /// Note: This is inefficient and should be avoided. Store the `TableRow`
    /// returned by `allocate()` instead.
    pub fn get_table_row(&self, key: Entity) -> Option<TableRow> {
        let index = self.entities.iter().position(|it| *it == key)?;
        Some(TableRow(index as u32))
    }

    /// Returns a reference to a column by its index.
    ///
    /// # Safety
    /// - `index` must be a valid column index obtained from `get_table_col()`
    #[inline(always)]
    pub unsafe fn get_column(&self, index: TableCol) -> &Column {
        debug_assert!((index.0 as usize) < self.columns.len());
        unsafe { self.columns.get_unchecked(index.0 as usize) }
    }

    /// Returns a mutable reference to a column by its index.
    ///
    /// # Safety
    /// - `index` must be a valid column index obtained from `get_table_col()`
    /// - No other references to the column may exist
    #[inline(always)]
    pub unsafe fn get_column_mut(&mut self, index: TableCol) -> &mut Column {
        debug_assert!((index.0 as usize) < self.columns.len());
        unsafe { self.columns.get_unchecked_mut(index.0 as usize) }
    }

    /// Returns a pointer to component data at the specified row and column.
    ///
    /// # Safety
    /// - `table_row` must be a valid row index
    /// - `table_col` must be a valid column index
    #[inline(always)]
    pub unsafe fn get_data(&self, table_row: TableRow, table_col: TableCol) -> Ptr<'_> {
        debug_assert!((table_row.0 as usize) < self.entity_count());
        unsafe {
            let col = self.get_column(table_col);
            col.get_data(table_row.0 as usize)
        }
    }

    /// Returns a pointer to component data at the specified column.
    ///
    /// # Safety
    /// - `table_col` must be a valid column index
    #[inline(always)]
    pub unsafe fn get_data_ptr(&self, table_col: TableCol) -> Ptr<'_> {
        unsafe {
            let col = self.get_column(table_col);
            col.get_data(0)
        }
    }

    /// Returns the added tick for a component at the specified row and column.
    ///
    /// # Safety
    /// - `table_row` must be a valid row index
    /// - `table_col` must be a valid column index
    #[inline(always)]
    pub unsafe fn get_added(&self, table_row: TableRow, table_col: TableCol) -> Tick {
        debug_assert!((table_row.0 as usize) < self.entity_count());
        unsafe {
            let col = self.get_column(table_col);
            col.get_added(table_row.0 as usize)
        }
    }

    /// Returns the changed tick for a component at the specified row and column.
    ///
    /// # Safety
    /// - `table_row` must be a valid row index
    /// - `table_col` must be a valid column index
    #[inline(always)]
    pub unsafe fn get_changed(&self, table_row: TableRow, table_col: TableCol) -> Tick {
        debug_assert!((table_row.0 as usize) < self.entity_count());
        unsafe {
            let col = self.get_column(table_col);
            col.get_changed(table_row.0 as usize)
        }
    }

    /// Returns a slice of component data for the entire column.
    ///
    /// # Safety
    /// - `table_col` must be a valid column index
    /// - The component type `T` must match the actual stored type
    /// - The returned slice is only valid while the table is not mutated
    #[inline(always)]
    pub unsafe fn get_data_slice<T>(&self, table_col: TableCol) -> &[T] {
        unsafe {
            let col = self.get_column(table_col);
            let ptr = col.get_data(0);
            ptr.debug_assert_aligned::<T>();
            let len = self.entity_count();
            let data = ptr.as_ptr() as *const T;
            slice::from_raw_parts(data, len)
        }
    }

    /// Returns a slice of added ticks for the entire column.
    ///
    /// # Safety
    /// - `table_col` must be a valid column index
    /// - The returned slice is only valid while the table is not mutated
    #[inline(always)]
    pub unsafe fn get_added_slice(&self, table_col: TableCol) -> &[Tick] {
        let len = self.entity_count();
        unsafe {
            let col = self.get_column(table_col);
            col.get_changed_slice().as_slice(len)
        }
    }

    /// Returns a slice of changed ticks for the entire column.
    ///
    /// # Safety
    /// - `table_col` must be a valid column index
    /// - The returned slice is only valid while the table is not mutated
    #[inline(always)]
    pub unsafe fn get_changed_slice(&self, table_col: TableCol) -> &[Tick] {
        let len = self.entity_count();
        unsafe {
            let col = self.get_column(table_col);
            col.get_changed_slice().as_slice(len)
        }
    }

    /// Returns an untyped reference to a component with change tracking.
    ///
    /// # Safety
    /// - `table_row` and `table_col` must be valid
    /// - The component must be initialized at the given row
    #[inline(always)]
    pub unsafe fn get_ref(
        &self,
        table_row: TableRow,
        table_col: TableCol,
        last_run: Tick,
        this_run: Tick,
    ) -> UntypedRef<'_> {
        debug_assert!((table_row.0 as usize) < self.entity_count());
        unsafe {
            let col = self.get_column(table_col);
            col.get_ref(table_row.0 as usize, last_run, this_run)
        }
    }

    /// Returns an untyped mutable reference to a component with change tracking.
    ///
    /// # Safety
    /// - `table_row` and `table_col` must be valid
    /// - The component must be initialized at the given row
    /// - No other references to the component may exist
    #[inline(always)]
    pub unsafe fn get_mut(
        &mut self,
        table_row: TableRow,
        table_col: TableCol,
        last_run: Tick,
        this_run: Tick,
    ) -> UntypedMut<'_> {
        debug_assert!((table_row.0 as usize) < self.entity_count());
        unsafe {
            let col = self.get_column_mut(table_col);
            col.get_mut(table_row.0 as usize, last_run, this_run)
        }
    }

    /// Returns an untyped slice reference to an entire column with change tracking.
    ///
    /// # Safety
    /// - `table_col` must be a valid column index
    /// - All components in the column must be initialized
    #[inline(always)]
    pub unsafe fn get_slice_ref(
        &self,
        table_col: TableCol,
        last_run: Tick,
        this_run: Tick,
    ) -> UntypedSliceRef<'_> {
        let len = self.entity_count();
        unsafe {
            let col = self.get_column(table_col);
            col.get_slice_ref(len, last_run, this_run)
        }
    }

    /// Returns an untyped mutable slice reference to an entire column with change tracking.
    ///
    /// # Safety
    /// - `table_col` must be a valid column index
    /// - All components in the column must be initialized
    /// - No other references to the column may exist
    #[inline(always)]
    pub unsafe fn get_slice_mut(
        &mut self,
        table_col: TableCol,
        last_run: Tick,
        this_run: Tick,
    ) -> UntypedSliceMut<'_> {
        let len = self.entity_count();
        unsafe {
            let col = self.get_column_mut(table_col);
            col.get_slice_mut(len, last_run, this_run)
        }
    }

    /// Initializes a component at the specified row.
    ///
    /// # Safety
    /// - `table_row` and `table_col` must be valid
    /// - The component slot must be uninitialized
    /// - `data` must point to valid data matching the column's type
    #[inline]
    pub unsafe fn init_item(
        &mut self,
        table_col: TableCol,
        table_row: TableRow,
        data: OwningPtr<'_>,
        tick: Tick,
    ) {
        debug_assert!((table_row.0 as usize) < self.entity_count());

        unsafe {
            let column = self.get_column_mut(table_col);
            column.init_item(table_row.0 as usize, data, tick);
        }
    }

    /// Replaces an existing component at the specified row.
    ///
    /// # Safety
    /// - `table_row` and `table_col` must be valid
    /// - The component slot must be initialized
    /// - `data` must point to valid data matching the column's type
    #[inline]
    pub unsafe fn replace_item(
        &mut self,
        table_col: TableCol,
        table_row: TableRow,
        data: OwningPtr<'_>,
        tick: Tick,
    ) {
        debug_assert!((table_row.0 as usize) < self.entity_count());

        unsafe {
            let column = self.get_column_mut(table_col);
            column.replace_item(table_row.0 as usize, data, tick);
        }
    }

    /// Removes a component and returns ownership of its data.
    ///
    /// # Safety
    /// - `table_row` and `table_col` must be valid
    /// - The component slot must be initialized
    /// - Caller must ensure the returned `OwningPtr` is properly handled
    #[inline]
    #[must_use = "The returned pointer should be used."]
    pub unsafe fn remove_item(
        &mut self,
        table_col: TableCol,
        table_row: TableRow,
    ) -> OwningPtr<'_> {
        debug_assert!((table_row.0 as usize) < self.entity_count());

        unsafe {
            let column = self.get_column_mut(table_col);
            column.remove_item(table_row.0 as usize)
        }
    }

    /// Updates change ticks for all components based on the provided check parameters.
    pub(crate) fn check_ticks(&mut self, check: CheckTicks) {
        let len = self.entity_count();
        self.columns.iter_mut().for_each(|c| unsafe {
            c.check_ticks(len, check);
        });
    }
}

// -----------------------------------------------------------------------------
// Move, Init data

/// Result of moving an entity between tables.
#[derive(Debug)]
pub struct TableMoveResult {
    pub new_row: TableRow,
    pub swapped: Option<Entity>,
}

impl Table {
    /// Removes an entity by swapping with the last row and dropping its components.
    ///
    /// # Safety
    /// - `table_row` must be a valid, initialized row
    /// - After this operation, the row is no longer valid
    pub unsafe fn swap_remove_and_drop(&mut self, table_row: TableRow) -> Option<MovedEntity> {
        let removal = table_row.0 as usize;
        let last = self.entity_count() - 1;
        debug_assert!(removal <= last);

        unsafe {
            if removal != last {
                let swapped = self.entities.move_last_to(last, removal);
                self.columns.iter_mut().for_each(|c| {
                    c.swap_drop_not_last(removal, last);
                });

                Some(MovedEntity::in_table(swapped, table_row))
            } else {
                self.entities.set_len(last);
                self.columns.iter_mut().for_each(|c| {
                    c.drop_item(last);
                });

                None
            }
        }
    }

    /// Removes an entity by swapping with the last row without dropping its components.
    ///
    /// # Safety
    /// - `table_row` must be a valid, initialized row
    /// - Caller must ensure components are properly handled elsewhere
    /// - After this operation, the row is no longer valid
    pub unsafe fn swap_remove_and_forget(&mut self, table_row: TableRow) -> Option<MovedEntity> {
        let removal = table_row.0 as usize;
        let last = self.entity_count() - 1;
        debug_assert!(removal <= last);

        unsafe {
            if removal != last {
                let swapped = self.entities.move_last_to(last, removal);
                self.columns.iter_mut().for_each(|c| {
                    c.swap_forget_not_last(removal, last);
                });

                Some(MovedEntity::in_table(swapped, table_row))
            } else {
                self.entities.set_len(last);
                // `Column::forget_item` do nothing.
                None
            }
        }
    }

    /// Moves an entity to another table, dropping components not present in the destination.
    ///
    /// # Safety
    /// - `table_row` must be a valid, initialized row in this table
    /// - `other` must be a valid table
    /// - Components are properly moved or dropped based on presence in destination
    pub unsafe fn move_to_and_drop_missing(
        &mut self,
        table_row: TableRow,
        other: &mut Table,
    ) -> Option<MovedEntity> {
        let src = table_row.0 as usize;
        let last = self.entity_count() - 1;
        debug_assert!(src <= last);

        unsafe {
            if src != last {
                let moved = *self.entities.get_unchecked(src);
                let swapped = self.entities.move_last_to(last, src);
                let new_row = other.allocate(moved);
                let dst = new_row.0 as usize;

                self.idents
                    .iter()
                    .zip(self.columns.iter_mut())
                    .for_each(|(&id, col)| {
                        if let Some(table_col) = other.get_table_col(id) {
                            let other_col = other.get_column_mut(table_col);
                            col.move_item_to(other_col, src, dst);
                            col.swap_forget_not_last(src, last);
                        } else {
                            col.swap_drop_not_last(src, last);
                        }
                    });

                Some(MovedEntity::in_table(swapped, new_row))
            } else {
                let moved = self.entities.remove_last(last);
                let new_row = other.allocate(moved);
                let dst = new_row.0 as usize;

                self.idents
                    .iter()
                    .zip(self.columns.iter_mut())
                    .for_each(|(&id, col)| {
                        if let Some(table_col) = other.get_table_col(id) {
                            let other_col = other.get_column_mut(table_col);
                            col.move_item_to(other_col, src, dst);
                        } else {
                            col.drop_item(last);
                        }
                    });

                None
            }
        }
    }

    /// Moves an entity to another table, forgetting components not present in the destination.
    ///
    /// # Safety
    /// - `table_row` must be a valid, initialized row in this table
    /// - `other` must be a valid table
    /// - Components not present in destination are forgotten (not dropped)
    /// - Caller must ensure forgotten components are handled elsewhere
    pub unsafe fn move_to_and_forget_missing(
        &mut self,
        table_row: TableRow,
        other: &mut Table,
    ) -> Option<MovedEntity> {
        let src = table_row.0 as usize;
        let last = self.entity_count() - 1;
        debug_assert!(src <= last);

        unsafe {
            if src != last {
                let moved = *self.entities.get_unchecked(src);
                let swapped = self.entities.move_last_to(last, src);
                let new_row = other.allocate(moved);
                let dst = new_row.0 as usize;

                self.idents
                    .iter()
                    .zip(self.columns.iter_mut())
                    .for_each(|(&id, col)| {
                        if let Some(table_col) = other.get_table_col(id) {
                            let other_col = other.get_column_mut(table_col);
                            col.move_item_to(other_col, src, dst);
                        }
                        col.swap_forget_not_last(src, last);
                    });

                Some(MovedEntity::in_table(swapped, new_row))
            } else {
                let moved = self.entities.remove_last(last);
                let new_row = other.allocate(moved);
                let dst = new_row.0 as usize;

                self.idents
                    .iter()
                    .zip(self.columns.iter_mut())
                    .for_each(|(&id, col)| {
                        if let Some(table_col) = other.get_table_col(id) {
                            let other_col = other.get_column_mut(table_col);
                            col.move_item_to(other_col, src, dst);
                        }
                    });

                None
            }
        }
    }
}
