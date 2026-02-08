#![allow(clippy::new_without_default, reason = "internal type")]

use alloc::boxed::Box;
use alloc::vec::Vec;
use core::fmt::Debug;

use vc_task::ComputeTaskPool;
use vc_utils::hash::HashMap;

use super::{Table, TableBuilder, TableId};

use crate::component::ComponentId;
use crate::tick::CheckTicks;

// -----------------------------------------------------------------------------
// Tables

pub struct Tables {
    tables: Vec<Table>,
    table_ids: HashMap<Box<[ComponentId]>, TableId>,
}

impl Tables {
    #[inline]
    pub(crate) fn new() -> Self {
        let mut tables: Vec<Table> = Vec::new();
        let mut table_ids: HashMap<Box<[ComponentId]>, TableId> = HashMap::new();

        tables.push(TableBuilder::new(0).build());
        table_ids.insert(Box::new([]), TableId::EMPTY);

        Tables { tables, table_ids }
    }

    pub fn check_ticks(&mut self, check: CheckTicks) {
        if let Some(task_pool) = ComputeTaskPool::try_get() {
            task_pool.scope(|scope| {
                for table in &mut self.tables {
                    scope.spawn(async move {
                        table.check_ticks(check);
                    });
                }
            });
        } else {
            for table in &mut self.tables {
                table.check_ticks(check);
            }
        }
    }

    #[inline(always)]
    pub unsafe fn get(&self, id: TableId) -> &Table {
        debug_assert!(id.index() < self.tables.len());
        unsafe { self.tables.get_unchecked(id.index()) }
    }

    #[inline(always)]
    pub unsafe fn get_mut(&mut self, id: TableId) -> &mut Table {
        debug_assert!(id.index() < self.tables.len());
        unsafe { self.tables.get_unchecked_mut(id.index()) }
    }
}

impl Debug for Tables {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_map()
            .entries(self.tables.iter().enumerate())
            .finish()
    }
}

// -----------------------------------------------------------------------------
// register

use crate::component::Components;
use crate::storage::StorageIndex;
use vc_utils::hash::hash_map::RawEntryMut;

impl Tables {
    pub(crate) fn register(
        &mut self,
        components: &Components,
        idents: &[ComponentId],
        indices: &mut [StorageIndex],
    ) -> TableId {
        debug_assert_eq!(idents.len(), indices.len());

        match self.table_ids.raw_entry_mut().from_key(idents) {
            RawEntryMut::Occupied(entry) => {
                let (_, &table_id) = entry.get_key_value();
                let table = unsafe { self.tables.get_unchecked_mut(table_id.index()) };

                idents
                    .iter()
                    .zip(indices.iter_mut())
                    .for_each(|(&id, index)| {
                        *index = unsafe { table.get_index(id) };
                    });

                table_id
            }
            RawEntryMut::Vacant(entry) => {
                let table_id = self.tables.len();
                assert!(table_id < u32::MAX as usize, "too many tables");
                let table_id = TableId::new(table_id as u32);

                let mut builder = TableBuilder::new(idents.len());

                idents
                    .iter()
                    .zip(indices.iter_mut())
                    .for_each(|(&id, index)| {
                        let info = unsafe { components.get(id) };
                        *index = unsafe { builder.insert(id, info.layout(), info.drop_fn()) };
                    });

                self.tables.push(builder.build());

                entry.insert(Box::from(idents), table_id);

                table_id
            }
        }
    }
}
