#![allow(clippy::missing_safety_doc, reason = "todo")]

mod comp_mut;
mod comp_raw;
mod comp_ref;
mod entity;
mod tuples;

// -----------------------------------------------------------------------------
// QueryData

use alloc::vec::Vec;

use crate::archetype::Archetype;
use crate::entity::Entity;
use crate::storage::{Table, TableRow};
use crate::system::{FilterData, FilterParamBuilder};
use crate::tick::Tick;
use crate::world::{UnsafeWorld, World, WorldMode};

/// Core trait for types that can be fetched from entities in a query.
///
/// This trait defines how a query accesses data from entities. It is implemented
/// for component references, tuples of components, and other data sources.
///
/// # Type Parameters
///
/// - [`State`](Self::State) - Static data shared across all query instances
/// - [`Cache`](Self::Cache) - Per-execution cached data for a specific world state
/// - [`Item`](Self::Item) - The type returned when fetching data for an entity
///
/// # Performance Considerations
///
/// - [`COMPONENTS_ARE_DENSE`](Self::COMPONENTS_ARE_DENSE) allows optimizations
///   when all accessed components use dense storage
/// - [`WORLD_MODE`](Self::WORLD_MODE) determines thread-safety requirements
/// - Cache methods enable pre-computation to speed up entity iteration
pub unsafe trait QueryData {
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

    /// The type returned when fetching data for a single entity.
    type Item<'world>;

    /// Indicates whether all components accessed by this filter use dense storage.
    ///
    /// - If `true`, the query can optimize by assuming components are stored in tables.
    /// - If `false`, the filter may access sparse components requiring map lookups.
    const COMPONENTS_ARE_DENSE: bool;

    /// Specifies the world access mode required by this data.
    const WORLD_MODE: WorldMode;

    /// Builds the static state for this query data.
    ///
    /// This is called once when the query is first created. The state is
    /// shared across all query executions and contains metadata needed for
    /// future cache building and fetching.
    ///
    /// # Safety
    /// - Must properly register all component accesses with the world
    /// - Must not modify world state in ways that violate invariants
    /// - Returned state must remain valid for the lifetime of the query
    unsafe fn build_state(world: &mut World) -> Self::State;

    /// Builds a per-execution cache for this query data.
    ///
    /// This is called at the beginning of each query execution to prepare
    /// world-specific data needed for fetching. The cache may contain direct
    /// pointers to component arrays or other performance-critical data.
    ///
    /// # Safety
    /// - The returned cache must remain valid for the duration of the query
    /// - World access must follow the provided tick parameters
    /// - Pointers stored in cache must remain valid while cache is alive
    unsafe fn build_cache<'w>(
        state: &Self::State,
        world: UnsafeWorld<'w>,
        last_run: Tick,
        this_run: Tick,
    ) -> Self::Cache<'w>;

    /// Builds filter parameters for query planning.
    ///
    /// This converts the query data into a list of [`FilterParamBuilder`]s that
    /// describe which components are accessed. These parameters are used by the
    /// query planner to construct efficient access patterns.
    ///
    /// # Safety
    /// - Must correctly represent all component accesses
    /// - Must not introduce conflicting access patterns
    /// - Must maintain correct order for component dependencies
    unsafe fn build_filter(state: &Self::State, out: &mut Vec<FilterParamBuilder>);

    /// Builds target information for data fetching.
    ///
    /// This configures how the fetched data will be stored or processed.
    /// Returns `true` if the target requires per-entity processing.
    ///
    /// # Safety
    /// - Target configuration must match actual data layout
    /// - Must correctly set access flags for change detection
    unsafe fn build_target(state: &Self::State, out: &mut FilterData) -> bool;

    /// Prepares the cache for a specific archetype.
    ///
    /// Called when the query begins processing a new archetype. The implementation
    /// can pre-compute archetype-specific information to speed up later fetching.
    ///
    /// # Safety
    /// - The archetype must remain valid for the duration of the query
    /// - Cache updates must not invalidate existing data
    /// - Must correctly handle archetype component layout
    unsafe fn set_for_arche<'w>(
        state: &Self::State,
        cache: &mut Self::Cache<'w>,
        arche: &'w Archetype,
        table: &'w Table,
    );

    /// Prepares the cache for a specific table.
    ///
    /// Called when the query begins processing a new table. The implementation
    /// can pre-compute table-specific information to speed up later fetching.
    ///
    /// # Safety
    /// - The table must remain valid for the duration of the query
    /// - Cache updates must not invalidate existing data
    /// - Must correctly handle table column layout
    unsafe fn set_for_table<'w>(state: &Self::State, cache: &mut Self::Cache<'w>, table: &'w Table);

    /// Fetches data for a single entity.
    ///
    /// This is called for each entity that passes all filter conditions.
    /// Returns `Some(item)` if the entity has the requested data, or `None`
    /// if the data is not available (for optional fetches).
    ///
    /// # Safety
    /// - The entity must exist and be valid
    /// - The table row must be valid for the current table
    /// - Cache must be properly set for the current archetype/table
    /// - Returned references must follow Rust's borrowing rules
    unsafe fn fetch<'w>(
        state: &Self::State,
        cache: &mut Self::Cache<'w>,
        entity: Entity,
        table_row: TableRow,
    ) -> Option<Self::Item<'w>>;
}

pub unsafe trait ReadOnlyQuery: QueryData {}
