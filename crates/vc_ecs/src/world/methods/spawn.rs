use vc_ptr::OwningPtr;
use vc_utils::extra::TypeIdMap;

use crate::bundle::{Bundle, BundleId};
use crate::component::ComponentWriter;
use crate::entity::{EntityLocation, SpawnError};
use crate::tick::Tick;
use crate::utils::DebugCheckedUnwrap;
use crate::world::{World, WorldEntityMut};

impl World {
    // We enable inlining to avoid copying data
    #[inline(always)]
    pub fn spawn<B: Bundle>(&mut self, bundle: B) -> WorldEntityMut<'_> {
        let bundle_id = self.register_bundle::<B>();

        vc_ptr::into_owning!(bundle);

        self.spawn_internal(bundle, bundle_id, B::write_fields, B::write_required)
    }

    #[inline(never)]
    fn spawn_internal(
        &mut self,
        data: OwningPtr<'_>,
        bundle_id: BundleId,
        write_fields: unsafe fn(&mut ComponentWriter, usize),
        write_required: unsafe fn(&mut ComponentWriter),
    ) -> WorldEntityMut<'_> {
        let tick = Tick::new(*self.this_run.get_mut());

        let arche_id = self.register_archetype(bundle_id);
        let archetype = unsafe { self.archetypes.get_unchecked_mut(arche_id) };

        let table_id = archetype.table_id;
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
                let _ = map.allocate(entity);
            });
        let table_row = unsafe { table.allocate(entity) };
        let arche_row = unsafe { archetype.allocate(entity) };

        unsafe {
            let mut writer = ComponentWriter {
                data,
                components,
                maps,
                table,
                entity,
                table_row,
                tick,
                writed: TypeIdMap::new(),
            };

            write_fields(&mut writer, 0);
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

        WorldEntityMut {
            world: self,
            entity,
            location,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::component::Component;
    use crate::world::{World, WorldIdAllocator};

    #[derive(Debug, PartialEq, Eq)]
    struct Foo;

    #[derive(Debug, PartialEq, Eq)]
    struct Bar(u64);

    unsafe impl Component for Foo {}
    unsafe impl Component for Bar {}

    #[test]
    fn spawn_basic() {
        let allocator = WorldIdAllocator::new();
        let mut world = World::new(allocator.alloc());

        let mut entity = world.spawn((Foo, Bar(123)));
        assert_eq!(entity.get::<Foo>().unwrap(), &Foo);
        assert_eq!(entity.get::<Bar>().unwrap(), &Bar(123));

        entity.get_mut::<Bar>().unwrap().0 = 321;
        assert_eq!(entity.get::<(Foo, Bar)>().unwrap(), (&Foo, &Bar(321)));

        // std::eprintln!("{world:?}");

        // std::eprintln!(
        //     "{entity}: ({:?} , {:?})",
        //     get_ref::<Foo>(&world, entity),
        //     get_ref::<Bar>(&world, entity),
        // );
    }
}
