use alloc::boxed::Box;
use alloc::vec::Vec;
use core::any::TypeId;

use vc_os::sync::Arc;
use vc_ptr::OwningPtr;

use super::World;
use crate::archetype::ArchetypeId;
use crate::bundle::{
    Bundle, BundleComponentRegistrar, BundleComponentWriter, BundleId, BundleInfo,
};
use crate::component::ComponentId;
use crate::entity::EntityLocation;
use crate::storage::{StorageIndex, StorageType};
use crate::tick::Tick;
use crate::world::EntityMut;

impl World {
    #[inline(always)]
    #[cfg_attr(any(debug_assertions, feature = "debug"), track_caller)]
    pub fn spawn<B: Bundle>(&mut self, bundle: B) -> EntityMut<'_> {
        let bundle_id = self.register_bundle::<B>();

        vc_ptr::into_owning!(bundle);

        self.spawn_internal(bundle, bundle_id, B::write_components)
    }

    #[inline(never)]
    fn spawn_internal(
        &mut self,
        data: OwningPtr<'_>,
        bundle_id: BundleId,
        write_fn: fn(usize, &mut BundleComponentWriter),
    ) -> EntityMut<'_> {
        let archetype_id = self.register_archetype(bundle_id);
        let archetype = unsafe { self.archetypes.get(archetype_id) };

        let table_id = archetype.table_id;
        let table = unsafe { self.storages.tables.get_mut(table_id) };

        let sparse_sets = &mut self.storages.sparse_sets;

        let components = &self.components;

        let entity = self.entity_allocator.alloc_mut();
        let entity_id = entity.id();
        let table_row = unsafe { table.allocate(entity) };

        let tick = Tick::new(*self.now_tick.get_mut());

        let mut writer = BundleComponentWriter {
            data,
            components,
            archetype,
            sparse_sets,
            table,
            table_row,
            entity_id,
            tick,
        };
        write_fn(0, &mut writer);

        let location = EntityLocation {
            archetype_id,
            table_id,
            table_row,
        };

        unsafe {
            self.entities.set_spawned(entity, location);
        }

        EntityMut {
            now_tick: Tick::new(*self.now_tick.get_mut()),
            world: self,
            entity,
            location,
        }
    }

    #[inline]
    fn register_bundle<B: Bundle>(&mut self) -> BundleId {
        if let Some(id) = self.bundles.get_id(TypeId::of::<B>()) {
            return id;
        }
        self.register_bundle_slow(
            TypeId::of::<B>(),
            B::COMPONENT_COUNT,
            B::register_components,
        )
    }

    #[cold]
    #[inline(never)]
    fn register_bundle_slow(
        &mut self,
        type_id: TypeId,
        component_count: usize,
        register_fn: fn(&mut BundleComponentRegistrar),
    ) -> BundleId {
        let mut buffer = Vec::with_capacity(component_count);

        let mut reg = BundleComponentRegistrar {
            components: &mut self.components,
            allocator: &mut self.compid_allocator,
            out: &mut buffer,
        };
        register_fn(&mut reg);

        let mut sparse_buf = buffer
            .extract_if(.., |id| unsafe {
                self.components.get(*id).storage_type() == StorageType::SparseSet
            })
            .collect::<Vec<_>>();

        // Remove duplicates and ensure orderliness
        buffer.sort_unstable();
        sparse_buf.sort_unstable();
        buffer.dedup();
        sparse_buf.dedup();

        // 0 < ComponentId <= u32::MAX, so buffer.len < u32
        let in_table = buffer.len() as u32;

        buffer.append(&mut sparse_buf);

        let components: Arc<[ComponentId]> = buffer.into();

        unsafe { self.bundles.register(type_id, components, in_table) }
    }

    #[inline]
    fn register_archetype(&mut self, bundle_id: BundleId) -> ArchetypeId {
        if let Some(id) = self.archetypes.get_id_by_bundle(bundle_id) {
            return id;
        }

        self.register_archetype_slow(bundle_id)
    }

    #[cold]
    #[inline(never)]
    fn register_archetype_slow(&mut self, bundle_id: BundleId) -> ArchetypeId {
        let info = unsafe { self.bundles.get_mut(bundle_id) };
        if let Some(archetype_id) = self.archetypes.get_id_by_components(&info.components) {
            self.archetypes.set_bundle_map(bundle_id, archetype_id);
            let archetype = unsafe { self.archetypes.get(archetype_id) };
            // The `Arc<[ComponentId]>` of BundleInfo is an independent memory,
            // Release it here to conserve resources
            info.components = archetype.components.clone();
            return archetype_id;
        }
        // Ensure immutability
        let info: &BundleInfo = info;

        let components: Arc<[ComponentId]> = info.components.clone();
        let mut storage_indices: Box<[StorageIndex]> =
            unsafe { Box::new_uninit_slice(components.len()).assume_init() };

        let table_id = {
            let idents: &[ComponentId] = &components[0..info.in_table as usize];
            let indices: &mut [StorageIndex] = &mut storage_indices[0..info.in_table as usize];
            self.storages
                .tables
                .register(&self.components, idents, indices)
        };

        {
            let idents: &[ComponentId] = &components[info.in_table as usize..];
            let indices: &mut [StorageIndex] = &mut storage_indices[info.in_table as usize..];
            self.storages
                .sparse_sets
                .register(&self.components, idents, indices);
        }

        unsafe {
            self.archetypes.register(
                bundle_id,
                table_id,
                info.in_table,
                components,
                storage_indices,
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::component::Component;
    use crate::storage::StorageType;
    use crate::world::{World, WorldId};

    #[derive(Debug, PartialEq, Eq)]
    struct Foo;

    #[derive(Debug, PartialEq, Eq)]
    struct Bar(u64);

    impl Component for Foo {
        const STORAGE_TYPE: StorageType = StorageType::SparseSet;
    }
    impl Component for Bar {}

    #[test]
    fn spawn_basic() {
        let mut world = World::new(WorldId::new(1));

        let entity = world.spawn((Foo, Bar(123)));
        assert_eq!(entity.get_ref::<Foo>().unwrap().into_inner(), &Foo);
        assert_eq!(entity.get_ref::<Bar>().unwrap().into_inner(), &Bar(123));

        // std::eprintln!("{world:?}");

        // std::eprintln!(
        //     "{entity}: ({:?} , {:?})",
        //     get_ref::<Foo>(&world, entity),
        //     get_ref::<Bar>(&world, entity),
        // );
    }
}
