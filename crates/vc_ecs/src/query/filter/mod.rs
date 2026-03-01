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

/// Core trait for types that can filter entities in a query.
///
/// # Safety
///
/// Implementing this trait requires careful attention to memory safety and
/// component access patterns. See trait methods for specific safety requirements.
pub unsafe trait QueryFilter {
    /// Static data shared across all query instances.
    ///
    /// This is typically built once during query construction and contains
    /// information like component IDs that don't change over the query's lifetime.
    type State: Send + Sync + 'static;

    /// Per-query cached data for a specific world state.
    ///
    /// This cache is rebuilt each time the query is executed and may contain
    /// world-specific data like component pointers or pre-computed lookup tables.
    type Cache<'world>;

    /// Indicates whether all components accessed by this filter use dense storage.
    ///
    /// - If `true`, the query can optimize by assuming components are stored in tables.
    /// - If `false`, the filter may access sparse components requiring map lookups.
    const COMPONENTS_ARE_DENSE: bool;

    /// Indicates whether this filter performs per-entity filtering.
    ///
    /// If `false`, the filter can be fully evaluated at the archetype/table level,
    /// allowing for optimizations like skipping the per-entity filter loop.
    const ENABLE_ENTITY_FILTER: bool;

    /// Builds the static state for this filter.
    ///
    /// This is called once when the query is first created. The state is
    /// shared across all query executions.
    ///
    /// # Safety
    /// - Implementations must properly register any component accesses
    /// - Must not modify world state in ways that violate invariants
    unsafe fn build_state(world: &mut World) -> Self::State;

    /// Builds a per-execution cache for this filter.
    ///
    /// This is called at the beginning of each query execution to prepare
    /// world-specific data needed for filtering.
    ///
    /// # Safety
    /// - The returned cache must remain valid for the duration of the query
    /// - World access must follow the provided tick parameters
    unsafe fn build_cache<'w>(
        state: &Self::State,
        world: UnsafeWorld<'w>,
        last_run: Tick,
        this_run: Tick,
    ) -> Self::Cache<'w>;

    /// Builds filter parameters for query planning.
    ///
    /// This converts the filter into a list of [`FilterParamBuilder`]s that
    /// are used to construct the final query access patterns.
    ///
    /// # Safety
    /// - Must correctly represent all component accesses
    /// - Must not introduce conflicting access patterns
    unsafe fn build_filter(state: &Self::State, out: &mut Vec<FilterParamBuilder>);

    /// Updates the cache for a specific archetype.
    ///
    /// Called when the query begins processing a new archetype. The filter
    /// can pre-compute archetype-level information to speed up later filtering.
    ///
    /// # Safety
    /// - The archetype must remain valid for the duration of the query
    /// - Cache updates must not invalidate existing data
    unsafe fn set_for_arche<'w>(
        state: &Self::State,
        cache: &mut Self::Cache<'w>,
        arche: &'w Archetype,
        table: &'w Table,
    );

    /// Updates the cache for a specific table.
    ///
    /// Called when the query begins processing a new table. The filter
    /// can pre-compute table-level information to speed up later filtering.
    ///
    /// # Safety
    /// - The table must remain valid for the duration of the query
    /// - Cache updates must not invalidate existing data
    unsafe fn set_for_table<'w>(state: &Self::State, cache: &mut Self::Cache<'w>, table: &'w Table);

    /// Performs per-entity filtering.
    ///
    /// This is called for each entity that passes archetype/table-level checks.
    /// Returns `true` if the entity should be included in query results.
    ///
    /// # Safety
    /// - The entity must exist and be valid
    /// - The table row must be valid for the current table
    /// - Cache data must be properly set for the current archetype/table
    unsafe fn filter<'w>(
        state: &Self::State,
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

    unsafe fn set_for_arche<'w>(
        _state: &Self::State,
        _cache: &mut Self::Cache<'w>,
        _arche: &'w Archetype,
        _table: &'w Table,
    ) {
    }

    unsafe fn set_for_table<'w>(
        _state: &Self::State,
        _cache: &mut Self::Cache<'w>,
        _table: &'w Table,
    ) {
    }

    unsafe fn filter<'w>(
        _state: &Self::State,
        _cache: &mut Self::Cache<'w>,
        _entity: Entity,
        _table_row: TableRow,
    ) -> bool {
        true
    }
}
