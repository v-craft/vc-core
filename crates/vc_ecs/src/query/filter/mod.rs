#![allow(clippy::needless_lifetimes, reason = "todo")]
#![allow(clippy::missing_safety_doc, reason = "todo")]

mod and;
mod changed;
mod or;
mod with;
mod without;

pub use and::And;
pub use changed::Changed;
pub use or::Or;
pub use with::With;
pub use without::Without;

// -----------------------------------------------------------------------------
// QueryFilter

use alloc::vec::Vec;

use crate::archetype::Archetype;
use crate::entity::Entity;
use crate::storage::{Table, TableRow};
use crate::system::FilterParamBuilder;
use crate::tick::Tick;
use crate::world::{UnsafeWorld, World};

pub unsafe trait QueryFilter {
    type State: Send + Sync + 'static;
    type Cache<'world>: Clone;

    const COMPONENTS_ARE_DENSE: bool;
    const ENABLE_ENTITY_FILTER: bool;

    unsafe fn build_state(world: &mut World) -> Self::State;

    unsafe fn build_cache<'w>(
        state: &Self::State,
        world: UnsafeWorld<'w>,
        last_run: Tick,
        this_run: Tick,
    ) -> Self::Cache<'w>;

    unsafe fn build_filter(state: &Self::State, outer: &mut Vec<FilterParamBuilder>);

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

    unsafe fn filter<'w, 's>(
        state: &'s Self::State,
        cache: &mut Self::Cache<'w>,
        entity: Entity,
        table_row: TableRow,
    ) -> bool;
}

// -----------------------------------------------------------------------------
// empty

unsafe impl QueryFilter for () {
    type State = ();
    type Cache<'world> = ();

    const COMPONENTS_ARE_DENSE: bool = true;
    const ENABLE_ENTITY_FILTER: bool = false;

    unsafe fn build_state(_world: &mut World) -> Self::State {}

    unsafe fn build_cache<'w>(
        _state: &Self::State,
        _world: UnsafeWorld<'w>,
        _last_run: Tick,
        _this_run: Tick,
    ) -> Self::Cache<'w> {
    }

    unsafe fn build_filter(_state: &Self::State, outer: &mut Vec<FilterParamBuilder>) {
        outer.push(FilterParamBuilder::new());
    }

    unsafe fn set_for_arche<'w, 's>(
        _state: &'s Self::State,
        _cache: &mut Self::Cache<'w>,
        _arche: &'w Archetype,
    ) {
    }

    unsafe fn set_for_table<'w, 's>(
        _state: &'s Self::State,
        _cache: &mut Self::Cache<'w>,
        _table: &'w Table,
    ) {
    }

    unsafe fn filter<'w, 's>(
        _state: &'s Self::State,
        _cache: &mut Self::Cache<'w>,
        _entity: Entity,
        _table_row: TableRow,
    ) -> bool {
        true
    }
}
