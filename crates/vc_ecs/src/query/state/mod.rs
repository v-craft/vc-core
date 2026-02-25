// use core::fmt::Debug;
// use alloc::vec::Vec;
// use alloc::boxed::Box;

// use vc_utils::hash::NoOpHashSet;

// use crate::archetype::ArcheId;
// use crate::storage::TableId;
// use crate::utils::DebugName;
// use crate::world::{World, WorldId, WorldMode};

// use crate::query::{QueryData, QueryFilter};
// use crate::system::{FilterData, FilterParam, FilterParamBuilder, SystemParam};

// #[derive(Clone, Copy)]
// pub union StorageId {
//     table_id: TableId,
//     arche_id: ArcheId,
// }

// pub struct QueryState<D: QueryData, F: QueryFilter = ()> {
//     pub world_id: WorldId,
//     pub version: usize,
//     pub storages: Vec<StorageId>,
//     pub is_dense: bool,
//     pub filter_data: FilterData,
//     pub filter_params: Box<[FilterParam]>,
//     pub d_state: D::State,
//     pub f_state: F::State,
// }

// impl<D: QueryData, F: QueryFilter> Debug for QueryState<D, F> {
//     fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
//         f.debug_struct("QueryState")
//             .field("world_id", &self.world_id)
//             .finish_non_exhaustive()
//     }
// }

// impl<D: QueryData, F: QueryFilter> QueryState<D, F> {
//     pub unsafe fn new(world: &mut World) -> Self {
//         let world_id = world.id();
//         let version = world.archetypes.version();
//         let is_dense = D::COMPONENTS_ARE_DENSE && F::COMPONENTS_ARE_DENSE;
//         let d_state = unsafe { D::build_state(world) };
//         let f_state = unsafe { F::build_state(world) };
//         let mut filter_data = FilterData::new();
//         unsafe {
//             if !D::filter_data(&d_state, &mut filter_data) {
//                 panic!("invalid query params: {}", DebugName::type_name::<QueryState<D, F>>());
//             }
//         }
//         let mut builders = Vec::<FilterParamBuilder>::new();
//         unsafe {
//             F::build_filter(&f_state, &mut builders);
//         }
//         let mut params: NoOpHashSet<FilterParam> = NoOpHashSet::new();
//         for builder in builders {
//             if let Some(param) = builder.build() {
//                 params.insert(param);
//             }
//         }
//         let filter_params: Box<[FilterParam]> = params.into_iter().collect();
//         if is_dense {

//         } else {

//         }
//         todo!()
//     }
// }
