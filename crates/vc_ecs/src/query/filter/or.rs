use alloc::vec::Vec;

use super::QueryFilter;
use crate::archetype::Archetype;
use crate::entity::Entity;
use crate::storage::{Table, TableRow};
use crate::system::FilterParamBuilder;
use crate::tick::Tick;
use crate::world::{UnsafeWorld, World};

// -----------------------------------------------------------------------------
// InOr

pub trait InOr {}

// -----------------------------------------------------------------------------
// Or

pub struct Or<T: InOr>(T);

// -----------------------------------------------------------------------------
// Or for Tuple

macro_rules! impl_tuple {
    (0 : []) => {};
    (1 : [ $index:tt : $name:ident ]) => {};
    ($num:literal : [$($index:tt : $name:ident),*]) => {
        impl<$($name: QueryFilter),*> InOr for ($($name),*) {}

        unsafe impl<$($name: QueryFilter),*> QueryFilter for Or<($($name),*)> {
            type State = ( $( <$name>::State ),* );
            type Cache<'world> = ( $( <$name>::Cache<'world> ),* );

            const COMPONENTS_ARE_DENSE: bool = {
                true $( && <$name>::COMPONENTS_ARE_DENSE )*
            };
            // The use of 'Or' may lead to incomplete archetype-based
            // filtering, thus entity filtering must be enabled.
            const ENABLE_ENTITY_FILTER: bool = true;

            unsafe fn build_state(
                world: &mut World,
            ) -> Self::State {
                unsafe {
                    ( $( <$name>::build_state(world), )* )
                }
            }

            unsafe fn build_cache<'w>(
                state: &Self::State,
                world: UnsafeWorld<'w>,
                last_run: Tick,
                this_run: Tick,
            ) -> Self::Cache<'w> {
                unsafe {
                    ( $( <$name>::build_cache(&state.$index, world, last_run, this_run), )* )
                }
            }

            unsafe fn build_filter(
                state: &Self::State,
                outer: &mut Vec<FilterParamBuilder>,
            ) {
                let mut ret = Vec::<FilterParamBuilder>::new();
                unsafe {
                    $( <$name>::build_filter(&state.$index, &mut ret); )*
                }
                outer.append(&mut ret);
            }


            unsafe fn set_for_arche<'w, 's>(
                state: &'s Self::State,
                cache: &mut Self::Cache<'w>,
                arche: &'w Archetype,
            ) {
                unsafe {
                    $( <$name>::set_for_arche(&state.$index, &mut cache.$index, arche); )*
                }
            }

            unsafe fn set_for_table<'w, 's>(
                state: &'s Self::State,
                cache: &mut Self::Cache<'w>,
                table: &'w Table,
            ) {
                unsafe {
                    $( <$name>::set_for_table(&state.$index, &mut cache.$index, table); )*
                }
            }

            unsafe fn filter<'w, 's>(
                state: &'s Self::State,
                cache: &mut Self::Cache<'w>,
                entity: Entity,
                table_row: TableRow,
            ) -> bool {
                unsafe {
                    false
                    $( || <$name>::filter(&state.$index, &mut cache.$index, entity, table_row) )*
                }
            }
        }
    };
}

vc_utils::range_invoke! {
    impl_tuple,  15: P
}
