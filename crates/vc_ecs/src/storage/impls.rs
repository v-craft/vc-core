use vc_task::ComputeTaskPool;

use crate::component::{ComponentInfo, ComponentStorage};
use crate::resource::ResourceInfo;
use crate::storage::{Maps, Tables};
use crate::tick::CheckTicks;

use super::ResSet;

#[derive(Debug)]
pub struct Storages {
    pub res_set: ResSet,
    pub tables: Tables,
    pub maps: Maps,
}

impl Storages {
    pub(crate) fn new() -> Storages {
        Storages {
            res_set: ResSet::new(),
            tables: Tables::new(),
            maps: Maps::new(),
        }
    }

    #[inline]
    pub fn prepare_resource(&mut self, info: &ResourceInfo) {
        self.res_set.prepare(info);
    }

    #[inline]
    pub fn prepare_component(&mut self, info: &ComponentInfo) {
        match info.storage() {
            ComponentStorage::Dense => {
                self.tables.prepare(info);
            }
            ComponentStorage::Sparse => {
                self.maps.prepare(info);
            }
        }
    }

    pub fn check_ticks(&mut self, check: CheckTicks) {
        let Storages {
            res_set,
            tables,
            maps,
        } = self;

        if let Some(task_pool) = ComputeTaskPool::try_get() {
            task_pool.scope(|scope| {
                scope.spawn(async move {
                    res_set.check_ticks(check);
                });
                tables.tables.iter_mut().for_each(|tb| {
                    scope.spawn(async move { tb.check_ticks(check) });
                });
                maps.maps.iter_mut().for_each(|mp| {
                    scope.spawn(async move { mp.check_ticks(check) });
                });
            });
        } else {
            res_set.check_ticks(check);
            tables.tables.iter_mut().for_each(|tb| {
                tb.check_ticks(check);
            });
            maps.maps.iter_mut().for_each(|mp| {
                mp.check_ticks(check);
            });
        }
    }
}
