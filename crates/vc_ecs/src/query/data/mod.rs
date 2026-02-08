#![allow(clippy::needless_lifetimes, reason = "todo")]
#![allow(clippy::missing_safety_doc, reason = "todo")]

mod entity;

// -----------------------------------------------------------------------------
// QueryData

use crate::archetype::Archetype;
use crate::entity::Entity;
use crate::storage::{Table, TableRow};
use crate::system::{FilterData, FilterParamBuilder};
use crate::tick::Tick;
use crate::world::{UnsafeWorld, World, WorldMode};
use alloc::vec::Vec;

pub unsafe trait QueryData {
    type State: Send + Sync + 'static;
    type Cache<'world>: Clone;
    type Item<'world>;

    const COMPONENTS_ARE_DENSE: bool;
    const MODE: WorldMode;

    unsafe fn build_state(world: &mut World) -> Self::State;

    unsafe fn build_cache<'w>(
        state: &Self::State,
        world: UnsafeWorld<'w>,
        last_run: Tick,
        this_run: Tick,
    ) -> Self::Cache<'w>;

    unsafe fn filter_param(state: &Self::State, out: &mut Vec<FilterParamBuilder>);
    unsafe fn filter_data(state: &Self::State, out: &mut FilterData) -> bool;

    unsafe fn set_for_arche<'w, 's>(
        state: &'s Self::State,
        cache: &mut Self::Cache<'w>,
        arche: &'w Archetype,
    );

    unsafe fn set_for_table<'w, 's>(
        state: &'s Self::State,
        cache: &mut Self::Cache<'w>,
        table: &'w Table,
    );

    unsafe fn fetch<'w, 's>(
        state: &'s Self::State,
        cache: &mut Self::Cache<'w>,
        entity: Entity,
        table_row: TableRow,
    ) -> Option<Self::Item<'w>>;
}
