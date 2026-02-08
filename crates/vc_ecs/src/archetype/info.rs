#![allow(clippy::new_without_default, reason = "internal type")]

use alloc::vec::Vec;
use core::fmt::Debug;

use vc_os::sync::Arc;

use crate::archetype::{ArcheId, ArcheRow};
use crate::component::ComponentId;
use crate::entity::{Entity, MovedEntity};
use crate::storage::TableId;

// -----------------------------------------------------------------------------
// Archetype

/// A collection of entities that share the exact same set of component types.
pub struct Archetype {
    pub(crate) id: ArcheId,
    pub(crate) table_id: TableId,
    // The number of components stored in the table
    pub(crate) dense_len: usize,
    // - `[0..dense_len]` stored in Tables
    // - `[dense_len..]` stored in Maps
    pub(crate) components: Arc<[ComponentId]>,
    // ArcheRow -> Entity
    pub(crate) entities: Vec<Entity>,
}

impl Debug for Archetype {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Archetype")
            .field("id", &self.id)
            .field("table_id", &self.table_id)
            .field("dense_components", &self.dense_components())
            .field("sparse_components", &self.sparse_components())
            .finish()
    }
}

impl Archetype {
    /// Returns the unique identifier of this archetype.
    #[inline(always)]
    pub fn id(&self) -> ArcheId {
        self.id
    }

    /// Returns the table ID where dense components are stored.
    #[inline(always)]
    pub fn table_id(&self) -> TableId {
        self.table_id
    }

    /// Returns the complete list of component types in this archetype.
    #[inline(always)]
    pub fn components(&self) -> &[ComponentId] {
        &self.components
    }

    /// Returns the list of dense component types (stored in tables).
    #[inline(always)]
    pub fn dense_components(&self) -> &[ComponentId] {
        &self.components[0..self.dense_len]
    }

    /// Returns the list of sparse component types (stored in maps).
    #[inline(always)]
    pub fn sparse_components(&self) -> &[ComponentId] {
        &self.components[self.dense_len..]
    }

    /// Checks if this archetype contains a specific component.
    ///
    /// # Complexity
    /// Time: O(logN), Space: O(1).
    #[inline]
    pub fn contains_component(&self, id: ComponentId) -> bool {
        self.contains_dense_component(id) || self.contains_sparse_component(id)
    }

    /// Checks if this archetype contains a dense component.
    ///
    /// # Complexity
    /// Time: O(logN), Space: O(1).
    #[inline]
    pub fn contains_dense_component(&self, id: ComponentId) -> bool {
        let components = self.dense_components();
        components.binary_search(&id).is_ok()
    }

    /// Checks if this archetype contains a sparse component.
    ///
    /// # Complexity
    /// Time: O(logN), Space: O(1).
    #[inline]
    pub fn contains_sparse_component(&self, id: ComponentId) -> bool {
        let components = self.sparse_components();
        components.binary_search(&id).is_ok()
    }

    #[inline(always)]
    pub fn entities(&self) -> &[Entity] {
        &self.entities
    }

    /// Allocates space for a new entity in this archetype.
    /// # Safety
    /// - The entity must not already exist in this archetype.
    /// - The caller must ensure that component storage (tables/maps) is
    ///   properly prepared for this entity.
    #[inline]
    pub unsafe fn allocate(&mut self, entity: Entity) -> ArcheRow {
        let row = ArcheRow(self.entities.len() as u32);
        self.entities.push(entity);
        row
    }

    /// Removes an entity from this archetype using swap-remove semantics.
    ///
    /// If the removed entity is not the last one, the last entity is moved
    /// into its position to maintain contiguity. Returns information about
    /// any moved entity that needs to be updated elsewhere.
    ///
    /// # Safety
    /// - The row must be valid and contain an entity.
    /// - The caller must update any references to moved entities.
    /// - Component data must be cleaned up from storage separately.
    pub unsafe fn swap_remove(&mut self, row: ArcheRow) -> Option<MovedEntity> {
        let last = self.entities.len() - 1;
        let dst = row.0 as usize;
        if dst == last {
            unsafe {
                self.entities.set_len(last);
            }
            None
        } else {
            let entity = unsafe { *self.entities.get_unchecked(last) };
            unsafe {
                *self.entities.get_unchecked_mut(dst) = entity;
                self.entities.set_len(last);
            }
            Some(MovedEntity::in_arche(entity, row))
        }
    }
}
