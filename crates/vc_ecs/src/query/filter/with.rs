use alloc::vec::Vec;

use super::QueryFilter;
use crate::archetype::Archetype;
use crate::component::{Component, ComponentId, ComponentStorage};
use crate::entity::Entity;
use crate::storage::{Table, TableRow};
use crate::system::FilterParamBuilder;
use crate::tick::Tick;
use crate::world::{UnsafeWorld, World};

// -----------------------------------------------------------------------------
// InWith

pub trait InWith {}

// -----------------------------------------------------------------------------
// With

pub struct With<T: InWith>(T);

// -----------------------------------------------------------------------------
// With for Component

#[cold]
#[inline(never)]
fn set_table_for_sparse() -> ! {
    unreachable!("Unexpected `set_for_table` for sparse component");
}

impl<T: Component> InWith for T {}

unsafe impl<T: Component> QueryFilter for With<T> {
    type State = ComponentId;
    type Cache<'world> = bool;

    const COMPONENTS_ARE_DENSE: bool = T::STORAGE.is_dense();
    const ENABLE_ENTITY_FILTER: bool = false;

    unsafe fn build_state(world: &mut World) -> Self::State {
        world.register_component::<T>()
    }

    unsafe fn build_cache<'w>(
        _state: &Self::State,
        _world: UnsafeWorld<'w>,
        _last_run: Tick,
        _this_run: Tick,
    ) -> Self::Cache<'w> {
        false
    }

    unsafe fn build_filter(state: &Self::State, outer: &mut Vec<FilterParamBuilder>) {
        let mut builder = FilterParamBuilder::new();
        builder.with(*state);
        outer.push(builder);
    }

    unsafe fn set_for_arche<'w, 's>(
        state: &'s Self::State,
        cache: &mut Self::Cache<'w>,
        arche: &'w Archetype,
    ) {
        match T::STORAGE {
            ComponentStorage::Dense => {
                *cache = arche.contains_dense_component(*state);
            }
            ComponentStorage::Sparse => {
                *cache = arche.contains_sparse_component(*state);
            }
        }
    }

    unsafe fn set_for_table<'w, 's>(
        state: &'s Self::State,
        cache: &mut Self::Cache<'w>,
        table: &'w Table,
    ) {
        match T::STORAGE {
            ComponentStorage::Dense => {
                *cache = table.get_table_col(*state).is_some();
            }
            ComponentStorage::Sparse => {
                // *cache = false;
                set_table_for_sparse();
            }
        }
    }

    unsafe fn filter<'w, 's>(
        _state: &'s Self::State,
        cache: &mut Self::Cache<'w>,
        _entity: Entity,
        _table_row: TableRow,
    ) -> bool {
        *cache
    }
}

// // -----------------------------------------------------------------------------
// // With for Tuple

macro_rules! to_component_id {
    ($_:ident) => {
        ComponentId
    };
}

macro_rules! impl_tuple {
    (0: []) => {};
    (1 : [ $index:tt : $name:ident ]) => {
        #[cfg_attr(docsrs, doc(fake_variadic))]
        impl<$name: Component> InWith for ($name,) {}

        #[cfg_attr(docsrs, doc(fake_variadic))]
        unsafe impl<$name: Component> QueryFilter for With<($name,)> {
            type State = ComponentId;
            type Cache<'world> = bool;

            const COMPONENTS_ARE_DENSE: bool = $name::STORAGE.is_dense();
            const ENABLE_ENTITY_FILTER: bool = false;

            unsafe fn build_state(
                world: &mut World,
            ) -> Self::State {
                world.register_component::<$name>()
            }

            unsafe fn build_cache<'w>(
                _state: &Self::State,
                _world: UnsafeWorld<'w>,
                _last_run: Tick,
                _this_run: Tick,
            ) -> Self::Cache<'w> {
                false
            }

            unsafe fn build_filter(
                state: &Self::State,
                outer: &mut Vec<FilterParamBuilder>,
            ) {
                let mut builder = FilterParamBuilder::new();
                builder.with(*state);
                outer.push(builder);
            }

            unsafe fn set_for_arche<'w, 's>(
                state: &'s Self::State,
                cache: &mut Self::Cache<'w>,
                arche: &'w Archetype,
            ) {
                match <$name>::STORAGE {
                    ComponentStorage::Dense => {
                        *cache = arche.contains_dense_component(*state);
                    },
                    ComponentStorage::Sparse => {
                        *cache = arche.contains_sparse_component(*state);
                    },
                }
            }

            unsafe fn set_for_table<'w, 's>(
                state: &'s Self::State,
                cache: &mut Self::Cache<'w>,
                table: &'w Table,
            ) {
                match <$name>::STORAGE {
                    ComponentStorage::Dense => {
                        *cache = table.get_table_col(*state).is_some();
                    },
                    ComponentStorage::Sparse => {
                        // *cache = false;
                        set_table_for_sparse();
                    },
                }
            }

            unsafe fn filter<'w, 's>(
                _state: &'s Self::State,
                cache: &mut Self::Cache<'w>,
                _entity: Entity,
                _table_row: TableRow,
            ) -> bool {
                *cache
            }
        }
    };
    ($num:literal : [$($index:tt : $name:ident),*]) => {
        #[cfg_attr(docsrs, doc(hidden))]
        impl<$($name: Component),*> InWith for ($($name),*) {}

        #[cfg_attr(docsrs, doc(hidden))]
        unsafe impl<$($name: Component),*> QueryFilter for With<($($name),*)> {
            type State = ( $( to_component_id!{ $name } ),* );
            type Cache<'world> = bool;

            const COMPONENTS_ARE_DENSE: bool = {
                true $( && <$name>::STORAGE.is_dense() )*
            };
            const ENABLE_ENTITY_FILTER: bool = false;

            unsafe fn build_state(world: &mut World) -> Self::State {
                ( $( world.register_component::<$name>(), )* )
            }

            unsafe fn build_cache<'w>(
                _state: &Self::State,
                _world: UnsafeWorld<'w>,
                _last_run: Tick,
                _this_run: Tick,
            ) -> Self::Cache<'w> {
                false
            }

            unsafe fn build_filter(
                state: &Self::State,
                outer: &mut Vec<FilterParamBuilder>,
            ) {
                let mut builder = FilterParamBuilder::new();
                $( builder.with(state.$index); )*
                outer.push(builder);
            }

            unsafe fn set_for_arche<'w, 's>(
                state: &'s Self::State,
                cache: &mut Self::Cache<'w>,
                arche: &'w Archetype,
            ) {
                *cache = true;
                $(
                    match <$name>::STORAGE {
                        ComponentStorage::Dense => {
                            *cache &= arche.contains_dense_component(state.$index);
                        },
                        ComponentStorage::Sparse => {
                            *cache &= arche.contains_sparse_component(state.$index);
                        },
                    }
                )*
            }

            unsafe fn set_for_table<'w, 's>(
                state: &'s Self::State,
                cache: &mut Self::Cache<'w>,
                table: &'w Table,
            ) {
                *cache = true;
                $(
                    match <$name>::STORAGE {
                        ComponentStorage::Dense => {
                            *cache &= table.get_table_col(state.$index).is_some();
                        },
                        ComponentStorage::Sparse => {
                            // *cache = false;
                            set_table_for_sparse();
                        },
                    }
                )*
            }

            unsafe fn filter<'w, 's>(
                _state: &'s Self::State,
                cache: &mut Self::Cache<'w>,
                _entity: Entity,
                _table_row: TableRow,
            ) -> bool {
                *cache
            }
        }
    };
}

vc_utils::range_invoke! {
    impl_tuple,  12: P
}
