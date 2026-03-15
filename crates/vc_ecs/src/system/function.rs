use core::marker::PhantomData;

use super::{AccessTable, System, SystemFlags, SystemIn, SystemMeta};
use crate::error::EcsError;
use crate::tick::Tick;
use crate::utils::DebugName;
use crate::world::{World, WorldId};

use super::{SystemInput, SystemParam};

// -----------------------------------------------------------------------------
// SystemFunction

type SystemInputData<'a, P> = <P as SystemInput>::Data<'a>;
type SystemParamItem<'w, 's, P> = <P as SystemParam>::Item<'w, 's>;

pub trait SystemFunction: Copy + Send + Sync + 'static {
    type Param: SystemParam;
    type Input: SystemInput;
    type Output;

    fn run(
        self,
        input: SystemInputData<Self::Input>,
        param: SystemParamItem<Self::Param>,
    ) -> Self::Output;
}

macro_rules! impl_tuple {
    (0: []) => {
        impl<O> SystemFunction for fn() -> O
        where
            O: 'static,
        {
            type Param = ();
            type Input = ();
            type Output = O;

            fn run(
                self,
                _input: (),
                _param: (),
            ) -> Self::Output {
                (self)()
            }
        }

        impl<I, O> SystemFunction for fn(SystemIn<I>) -> O
        where
            O: 'static,
            I: SystemInput + 'static,
            Self: Fn(SystemIn<I::Item<'_>>) -> O,
        {
            type Param = ();
            type Input = I;
            type Output = O;

            fn run(
                self,
                input: I::Data<'_>,
                _param: (),
            ) -> Self::Output {
                #[inline(always)]
                fn call<I, O>(
                    func: impl Fn(I) -> O,
                    input: I,
                ) -> O {
                    func(input)
                }

                call(self, SystemIn(I::wrap(input)))
            }
        }
    };
    (1 : [ $index:tt : $name:ident ]) => {
        #[cfg_attr(docsrs, doc(fake_variadic))]
        #[cfg_attr(docsrs, doc = "This trait is implemented for tuples up to 12 items long.")]
        impl<O, $name> SystemFunction for fn($name) -> O
        where
            O: 'static,
            $name: SystemParam + 'static,
            Self: Fn(<$name>::Item<'_, '_>) -> O,
        {
            type Param = ( $name, );
            type Input = ();
            type Output = O;

            fn run(
                self,
                _input: (),
                param: ( <$name>::Item<'_,'_> ,),
            ) -> Self::Output {
                #[inline(always)]
                fn call<O, $name>(
                    func: impl Fn($name) -> O,
                    param: ( $name , ),
                ) -> O {
                    func(param.0)
                }

                call(self, param)
            }
        }

        #[cfg_attr(docsrs, doc(fake_variadic))]
        #[cfg_attr(docsrs, doc = "This trait is implemented for tuples up to 12 items long.")]
        impl<I, O, $name> SystemFunction for fn(SystemIn<I>, $name) -> O
        where
            O: 'static,
            I: SystemInput + 'static,
            $name: SystemParam + 'static,
            Self: Fn(SystemIn<I::Item<'_>>, <$name>::Item<'_, '_>) -> O,
        {
            type Param = ( $name, );
            type Input = I;
            type Output = O;

            fn run(
                self,
                input: I::Data<'_>,
                param: ( <$name>::Item<'_,'_> ,),
            ) -> Self::Output {
                #[inline(always)]
                fn call<I, O, $name>(
                    func: impl Fn(I, $name) -> O,
                    input: I,
                    param: ( $name , ),
                ) -> O {
                    func(input, param.0)
                }

                call(self, SystemIn(I::wrap(input)), param)
            }
        }
    };
    ($num:literal : [$($index:tt : $name:ident),*]) => {
        #[cfg_attr(docsrs, doc(hidden))]
        impl<O, $($name),*> SystemFunction for fn($($name),*) -> O
        where
            O: 'static,
            $($name: SystemParam + 'static,)*
            Self: Fn($(<$name>::Item<'_, '_>),*) -> O,
        {
            type Param = ( $($name),* );
            type Input = ();
            type Output = O;

            fn run(
                self,
                _input: (),
                param: ( $(<$name>::Item<'_,'_>, )* ),
            ) -> Self::Output {
                #[inline(always)]
                fn call<O, $($name),*>(
                    func: impl Fn($($name),*) -> O,
                    param: ( $($name),* ),
                ) -> O {
                    func($(param.$index),*)
                }

                call(self, param)
            }
        }

        #[cfg_attr(docsrs, doc(hidden))]
        impl<I, O, $($name),*> SystemFunction for fn(SystemIn<I>, $($name),*) -> O
        where
            O: 'static,
            I: SystemInput + 'static,
            $($name: SystemParam + 'static,)*
            Self: Fn(SystemIn<I::Item<'_>>, $(<$name>::Item<'_, '_>),*) -> O,
        {
            type Param = ( $($name),* );
            type Input = I;
            type Output = O;

            fn run(
                self,
                input: I::Data<'_>,
                param: ( $(<$name>::Item<'_,'_>, )* ),
            ) -> Self::Output {
                #[inline(always)]
                fn call<I, O, $($name),*>(
                    func: impl Fn(I, $($name),*) -> O,
                    input: I,
                    param: ( $($name),* ),
                ) -> O {
                    func(input, $(param.$index),*)
                }

                call(self, SystemIn(I::wrap(input)), param)
            }
        }
    }
}

vc_utils::range_invoke!(impl_tuple, 12);

// -----------------------------------------------------------------------------
// FunctionSystem

struct FunctionState<P: SystemParam> {
    param: P::State,
    world_id: WorldId,
}

pub struct FunctionSystem<I, O, F: SystemFunction> {
    func: F,
    meta: SystemMeta,
    state: Option<FunctionState<F::Param>>,
    _marker: PhantomData<fn(I) -> O>,
}

impl<I, O, F> Clone for FunctionSystem<I, O, F>
where
    F: SystemFunction,
{
    fn clone(&self) -> Self {
        Self {
            func: self.func,
            state: None,
            meta: SystemMeta::new::<F>(),
            _marker: PhantomData,
        }
    }
}

impl<I, O, F> FunctionSystem<I, O, F>
where
    F: SystemFunction,
{
    pub fn new(func: F) -> Self {
        let mut meta = SystemMeta::new::<F>();
        if <F::Param as SystemParam>::EXCLUSIVE {
            meta.set_exclusive();
        }
        if <F::Param as SystemParam>::NON_SEND {
            meta.set_non_send();
        }

        Self {
            func,
            meta,
            state: None,
            _marker: PhantomData,
        }
    }
}

impl<I, O, F> System for FunctionSystem<I, O, F>
where
    I: 'static,
    O: 'static,
    F: SystemFunction + 'static,
{
    type Input = F::Input;
    type Output = F::Output;

    fn name(&self) -> DebugName {
        self.meta.name()
    }

    fn flags(&self) -> SystemFlags {
        self.meta.flags()
    }

    fn get_last_run(&self) -> Tick {
        self.meta.get_last_run()
    }

    fn set_last_run(&mut self, last_run: Tick) {
        self.meta.set_last_run(last_run)
    }

    fn initialize(&mut self, world: &mut World) -> AccessTable {
        let mut table = AccessTable::new();
        let state = self.state.get_or_insert_with(|| FunctionState {
            param: <F::Param as SystemParam>::init_state(world),
            world_id: world.id(),
        });
        let validity = <F::Param as SystemParam>::mark_access(&mut table, &state.param);
        assert!(validity, "invalid system {}", self.meta.name());
        table
    }

    unsafe fn run(
        &mut self,
        input: <Self::Input as SystemInput>::Data<'_>,
        world: crate::world::UnsafeWorld<'_>,
    ) -> Result<Self::Output, EcsError> {
        let Some(state) = &mut self.state else {
            todo!()
        };
        assert_eq!(unsafe { world.read_only().id() }, state.world_id);

        let last_run = self.meta.get_last_run();
        let this_run = unsafe { world.read_only().advance_tick() };
        let param = unsafe {
            <F::Param as SystemParam>::get_param(world, &mut state.param, last_run, this_run)
        };
        let output = <F as SystemFunction>::run(self.func, input, param);
        self.meta.set_last_run(this_run);
        Ok(output)
    }
}
