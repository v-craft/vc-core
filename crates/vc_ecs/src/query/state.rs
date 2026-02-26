#![allow(clippy::needless_lifetimes, reason = "todo")]
#![allow(clippy::missing_safety_doc, reason = "todo")]

use alloc::boxed::Box;
use alloc::collections::BTreeSet;
use alloc::vec::Vec;
use core::fmt::Debug;

use vc_utils::hash::NoOpHashSet;

use crate::archetype::{ArcheFilter, ArcheId, Archetypes};
use crate::storage::TableId;
use crate::utils::DebugName;
use crate::world::{World, WorldId};

use crate::query::{QueryData, QueryFilter};
use crate::system::{AccessTable, FilterData, FilterParam, FilterParamBuilder};

#[derive(Clone, Copy)]
pub union StorageId {
    pub table_id: TableId,
    pub arche_id: ArcheId,
}

const _STATIC_ASSERT_: () = const {
    assert!(size_of::<ArcheId>() == size_of::<StorageId>());
    assert!(size_of::<TableId>() == size_of::<StorageId>());
    assert!(align_of::<ArcheId>() == align_of::<StorageId>());
    assert!(align_of::<TableId>() == align_of::<StorageId>());
};

pub struct QueryState<D: QueryData, F: QueryFilter = ()> {
    pub(crate) world_id: WorldId,
    pub(crate) version: usize,
    pub(crate) storages: Vec<StorageId>,
    pub(crate) is_dense: bool,
    pub(crate) filter_data: FilterData,
    pub(crate) filter_params: Box<[FilterParam]>,
    pub(crate) d_state: D::State,
    pub(crate) f_state: F::State,
}

impl<D: QueryData, F: QueryFilter> Debug for QueryState<D, F> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("QueryState")
            .field("world_id", &self.world_id)
            .finish_non_exhaustive()
    }
}

impl<D: QueryData, F: QueryFilter> QueryState<D, F> {
    pub const IS_DENSE: bool = D::COMPONENTS_ARE_DENSE && F::COMPONENTS_ARE_DENSE;

    pub unsafe fn init(world: &mut World) -> Self {
        let world_id = world.id();
        let version = world.archetypes.len();
        let is_dense = D::COMPONENTS_ARE_DENSE && F::COMPONENTS_ARE_DENSE;
        let d_state = unsafe { D::build_state(world) };
        let f_state = unsafe { F::build_state(world) };
        let mut filter_data = FilterData::new();
        unsafe {
            if !D::build_target(&d_state, &mut filter_data) {
                panic!(
                    "invalid query params: {}",
                    DebugName::type_name::<QueryState<D, F>>()
                );
            }
        }
        let mut builders = Vec::<FilterParamBuilder>::new();
        unsafe {
            F::build_filter(&f_state, &mut builders);
            D::build_filter(&d_state, &mut builders);
        }

        let filter_params: Box<[FilterParam]> = collect_param(builders);

        let arche_filter = world.archetypes.filter();
        let arches: Vec<ArcheId> = collect_arches(&filter_params, &arche_filter);

        let storages: Vec<StorageId> = if is_dense {
            collect_tables(arches, &world.archetypes)
        } else {
            unsafe { core::mem::transmute::<Vec<ArcheId>, Vec<StorageId>>(arches) }
        };

        QueryState {
            world_id,
            version,
            storages,
            is_dense,
            filter_data,
            filter_params,
            d_state,
            f_state,
        }
    }

    pub unsafe fn updata(&mut self, world: &World) {
        debug_assert!(self.world_id == world.id());

        let archetypes = &world.archetypes;
        if archetypes.len() > self.version {
            updata_state(
                self.is_dense,
                &mut self.version,
                &mut self.storages,
                &self.filter_params,
                archetypes,
            );
        }
    }

    pub fn mark_assess(&self, access_table: &mut AccessTable) -> bool {
        let data: &FilterData = &self.filter_data;
        let params: &[FilterParam] = &self.filter_params;
        if access_table.can_query(data, params) {
            access_table.set_query(data, params);
            true
        } else {
            false
        }
    }
}

#[inline(never)]
fn updata_state(
    is_dense: bool,
    version: &mut usize,
    storages: &mut Vec<StorageId>,
    filter_params: &[FilterParam],
    archetypes: &Archetypes,
) {
    if is_dense {
        updata_dense_state(version, storages, filter_params, archetypes);
    } else {
        updata_sparse_state(version, storages, filter_params, archetypes);
    }
}

#[inline(never)]
fn updata_dense_state(
    version: &mut usize,
    storages: &mut Vec<StorageId>,
    filter_params: &[FilterParam],
    archetypes: &Archetypes,
) {
    let new_version = archetypes.len();

    let old_tables: &[StorageId] = &storages[..];
    let old_tables: &[TableId] = unsafe { core::mem::transmute(old_tables) };

    let mut new_tables: Vec<StorageId> = Vec::with_capacity(new_version - *version);
    for arche_id in (*version)..new_version {
        let arche_id = unsafe { ArcheId::new_unchecked(arche_id as u32) };
        let archetype = unsafe { archetypes.get_unchecked(arche_id) };
        let table_id = archetype.table_id();

        if old_tables.binary_search(&table_id).is_err() {
            let matched = filter_params
                .iter()
                .any(|param| archetype.matches_sorted(param.with(), param.without()));
            if matched {
                new_tables.push(StorageId { table_id });
            }
        }
    }
    storages.append(&mut new_tables);

    let tables: &mut Vec<TableId> = unsafe { core::mem::transmute(storages) };
    tables.sort();
    tables.dedup();

    *version = new_version;
}

fn updata_sparse_state(
    version: &mut usize,
    storages: &mut Vec<StorageId>,
    filter_params: &[FilterParam],
    archetypes: &Archetypes,
) {
    let new_version = archetypes.len();
    for arche_id in (*version)..new_version {
        let arche_id = unsafe { ArcheId::new_unchecked(arche_id as u32) };
        let archetype = unsafe { archetypes.get_unchecked(arche_id) };
        let matched = filter_params
            .iter()
            .any(|param| archetype.matches_sorted(param.with(), param.without()));
        if matched {
            storages.push(StorageId { arche_id });
        }
    }
    *version = new_version;
}

#[inline(never)]
fn collect_param(builders: Vec<FilterParamBuilder>) -> Box<[FilterParam]> {
    // We use NoOpHash because FilterParam is pre-hased.
    let mut params: NoOpHashSet<FilterParam> = NoOpHashSet::new();
    builders.into_iter().for_each(|builder| {
        if let Some(param) = builder.build() {
            params.insert(param);
        }
    });

    params.into_iter().collect()
}

#[inline(never)]
fn collect_arches(params: &[FilterParam], arche_filter: &ArcheFilter<'_>) -> Vec<ArcheId> {
    // N: the number of archetypes
    // M: the average number of components in an achetype
    // X: the number of filter_params
    // Y: the average number of components in a filter_param

    // Collect From ArcheFilter: X * Y * F(N, M)
    // Collect From Each Arche : X * Y * M * log N

    // We hope the results are in order.
    let mut collector = BTreeSet::<ArcheId>::new();

    params.iter().for_each(|param| {
        let mut filter = arche_filter.clone();
        param.with().iter().for_each(|id| {
            filter.with(*id);
        });
        param.without().iter().for_each(|id| {
            filter.without(*id);
        });
        filter.collect_to(&mut collector);
    });

    collector.into_iter().collect()
}

#[inline(never)]
fn collect_tables(idents: Vec<ArcheId>, archetypes: &Archetypes) -> Vec<StorageId> {
    // We hope the results are in order.
    let mut tables = BTreeSet::<TableId>::new();

    idents.into_iter().for_each(|arche_id| unsafe {
        let arche = archetypes.get_unchecked(arche_id);
        tables.insert(arche.table_id());
    });

    tables
        .into_iter()
        .map(|table_id| StorageId { table_id })
        .collect()
}
