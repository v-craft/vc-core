use vc_ptr::OwningPtr;

use crate::archetype::ArcheId;
use crate::bundle::{Bundle, BundleId};
use crate::component::ComponentWriter;
use crate::entity::{EntityLocation, SpawnError};
use crate::tick::Tick;
use crate::utils::DebugCheckedUnwrap;
use crate::world::{EntityOwned, World};

impl World {
    // We enable inlining to avoid copying data
    #[inline(always)]
    pub fn spawn<B: Bundle>(&mut self, bundle: B) -> EntityOwned<'_> {
        let bundle_id = self.register_bundle::<B>();

        vc_ptr::into_owning!(bundle);

        self.spawn_internal(bundle, bundle_id, B::write_explicit, B::write_required)
    }

    #[inline(never)]
    fn spawn_internal(
        &mut self,
        data: OwningPtr<'_>,
        bundle_id: BundleId,
        write_explicit: unsafe fn(&mut ComponentWriter, usize),
        write_required: unsafe fn(&mut ComponentWriter),
    ) -> EntityOwned<'_> {
        let tick = Tick::new(*self.this_run.get_mut());

        let arche_id = self.register_archetype_by_bundle(bundle_id);
        let archetype = unsafe { self.archetypes.get_unchecked_mut(arche_id) };

        let table_id = archetype.table_id();
        let table = unsafe { self.storages.tables.get_unchecked_mut(table_id) };

        let maps = &mut self.storages.maps;
        let components = &self.components;

        let mut entity = self.allocator.alloc_mut();
        if let Err(e) = self.entities.can_spawned(entity) {
            if let SpawnError::Mismatch { .. } = SpawnError::from(e) {
                entity = unsafe { self.entities.free(entity.id(), 1) };
                log::warn!("spawning with a unexpected Entity: {entity}");
            } else {
                e.handle_error();
            }
        }

        archetype
            .sparse_components()
            .iter()
            .for_each(|&cid| unsafe {
                let map_id = maps.get_id(cid).debug_checked_unwrap();
                let map = maps.get_unchecked_mut(map_id);
                let _ = map.alloc(entity); // `MapRow` may be cached in the future.
            });
        let table_row = unsafe { table.allocate(entity) };
        let arche_row = unsafe { archetype.insert_entity(entity) };

        unsafe {
            let mut writer =
                ComponentWriter::new(data, entity, table_row, tick, maps, table, components);

            write_explicit(&mut writer, 0);
            write_required(&mut writer);
        }

        let location = EntityLocation {
            arche_id,
            arche_row,
            table_id,
            table_row,
        };

        unsafe {
            self.entities.set_spawned(entity, location).unwrap();
        }

        EntityOwned {
            world: self.unsafe_world(),
            entity,
            location,
        }
    }

    #[inline]
    fn register_archetype_by_bundle(&mut self, bundle_id: BundleId) -> ArcheId {
        if let Some(id) = self.archetypes.get_id_by_bundle(bundle_id) {
            id
        } else {
            self.register_archetype_by_bundle_slow(bundle_id)
        }
    }

    #[cold]
    #[inline(never)]
    fn register_archetype_by_bundle_slow(&mut self, bundle_id: BundleId) -> ArcheId {
        let info = self.bundles.get(bundle_id).unwrap();
        if let Some(id) = self.archetypes.get_id(info.components()) {
            unsafe {
                self.archetypes.set_bundle_map(bundle_id, id);
            }
            return id;
        }

        let dense_len = info.dense_len();
        let components = info.clone_components();
        let table_id = unsafe {
            let sparses = info.sparse_components();
            self.storages.maps.register(&self.components, sparses);
            let denses = info.dense_components();
            self.storages.tables.register(&self.components, denses)
        };

        unsafe {
            let id = self.archetypes.register(table_id, dense_len, components);
            self.archetypes.set_bundle_map(bundle_id, id);
            id
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::component::{Component, ComponentStorage};
    use crate::world::{World, WorldIdAllocator};
    use alloc::string::String;

    #[derive(Debug, PartialEq, Eq)]
    struct Foo;

    #[derive(Debug, PartialEq, Eq)]
    struct Bar(u64);

    #[derive(Debug, PartialEq, Eq)]
    struct Baz(String);

    unsafe impl Component for Foo {}
    unsafe impl Component for Bar {}
    unsafe impl Component for Baz {
        const STORAGE: ComponentStorage = ComponentStorage::Sparse;
    }

    #[test]
    fn spawn_basic() {
        let allocator = WorldIdAllocator::new();
        let mut world = World::new(allocator.alloc());

        let mut entity = world.spawn((Foo, Bar(123), Baz(String::from("hello"))));
        assert_eq!(entity.get::<Foo>().unwrap(), &Foo);
        assert_eq!(entity.get::<Bar>().unwrap(), &Bar(123));

        entity.get_mut::<Bar>().unwrap().0 = 321;
        assert_eq!(entity.get::<(Foo, Bar)>().unwrap(), (&Foo, &Bar(321)));

        let baz = entity.get::<Baz>().unwrap();
        assert_eq!(&baz.0, "hello");

        // std::eprintln!("{world:?}");

        // std::eprintln!(
        //     "{entity}: ({:?} , {:?})",
        //     get_ref::<Foo>(&world, entity),
        //     get_ref::<Bar>(&world, entity),
        // );
    }
}
