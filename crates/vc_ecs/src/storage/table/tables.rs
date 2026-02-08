#![expect(unsafe_code, reason = "get_unchecked is unsafe")]

use alloc::boxed::Box;
use alloc::vec::Vec;
use core::ops::{Index, IndexMut};

use vc_utils::hash::HashMap;

use super::{Table, TableId};

use crate::component::ComponentId;
use crate::storage::TableBuilder;
use crate::tick::CheckTicks;

// -----------------------------------------------------------------------------
// Tables

pub struct Tables {
    tables: Vec<Table>,
    table_ids: HashMap<Box<[ComponentId]>, TableId>,
}

impl Index<TableId> for Tables {
    type Output = Table;

    #[inline]
    fn index(&self, index: TableId) -> &Self::Output {
        &self.tables[index.index()]
    }
}

impl IndexMut<TableId> for Tables {
    #[inline]
    fn index_mut(&mut self, index: TableId) -> &mut Self::Output {
        &mut self.tables[index.index()]
    }
}

impl Tables {
    #[inline]
    pub fn empty() -> Self {
        let mut tables: Vec<Table> = Vec::new();
        let mut table_ids: HashMap<Box<[ComponentId]>, TableId> = HashMap::new();

        tables.push(TableBuilder::new(0).build());
        table_ids.insert(Box::new([]), TableId::EMPTY);

        Tables { tables, table_ids }
    }

    #[inline]
    pub fn table_count(&self) -> usize {
        self.tables.len()
    }

    #[inline]
    pub fn iter(&self) -> impl ExactSizeIterator<Item = (TableId, &Table)> {
        self.tables
            .iter()
            .enumerate()
            .map(|(id, table)| (TableId::new(id as u32), table))
    }

    #[inline]
    pub fn iter_mut(&mut self) -> impl ExactSizeIterator<Item = (TableId, &mut Table)> {
        self.tables
            .iter_mut()
            .enumerate()
            .map(|(id, table)| (TableId::new(id as u32), table))
    }

    #[inline]
    pub fn clear_entities(&mut self) {
        for table in &mut self.tables {
            table.dealloc();
        }
    }

    #[inline]
    pub fn check_ticks(&mut self, check: CheckTicks) {
        for table in &mut self.tables {
            table.check_ticks(check);
        }
    }

    #[inline(always)]
    pub unsafe fn get_unchecked(&self, id: TableId) -> &Table {
        unsafe { self.tables.get_unchecked(id.index()) }
    }

    #[inline(always)]
    pub unsafe fn get_unchecked_mut(&mut self, id: TableId) -> &mut Table {
        unsafe { self.tables.get_unchecked_mut(id.index()) }
    }

    #[inline(always)]
    pub unsafe fn get_unchecked_mut_2(
        &mut self,
        a: TableId,
        b: TableId,
    ) -> (&mut Table, &mut Table) {
        // A manually implementation of `get_disjoint_unchecked_mut`.
        let base_ptr = self.tables.as_mut_ptr();
        unsafe { (&mut *base_ptr.add(a.index()), &mut *base_ptr.add(b.index())) }
    }
}

// -----------------------------------------------------------------------------
// Create Table From Components

use crate::component::Components;
use crate::utils::DebugCheckedUnwrap;

impl Tables {
    pub unsafe fn get_id_and_raw_indecies_or_insert(
        &mut self,
        ids: &[ComponentId],
        components: &Components,
    ) -> (TableId, Box<[u32]>) {
        use vc_utils::hash::hash_map::RawEntryMut;

        let tables = &mut self.tables;

        let raw_entry = self.table_ids.raw_entry_mut().from_key(ids);

        match raw_entry {
            RawEntryMut::Occupied(entry) => {
                let table_id = *entry.into_key_value().1;
                let table = &mut tables[table_id.index()];

                let mut raw_indecies = Vec::<u32>::with_capacity(ids.len());
                for &id in ids {
                    raw_indecies.push(unsafe { table.get_raw_index(id).debug_checked_unwrap() });
                }

                (table_id, raw_indecies.into_boxed_slice())
            }
            RawEntryMut::Vacant(entry) => {
                assert!(tables.len() <= u32::MAX as usize, "too many tables");

                let table_id = TableId::new(tables.len() as u32);

                let mut table = TableBuilder::new(ids.len());

                let mut raw_indecies = Vec::<u32>::with_capacity(ids.len());

                for &id in ids {
                    let info = unsafe { components.get_info_unchecked(id) };
                    let raw_index = unsafe { table.insert(id, info.layout(), info.drop_fn()) };
                    raw_indecies.push(raw_index);
                }

                tables.push(table.build());
                entry.insert(ids.into(), table_id);

                (table_id, raw_indecies.into_boxed_slice())
            }
        }
    }
}
