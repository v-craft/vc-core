use core::marker::PhantomData;

use crate::world::WorldId;
use super::{SystemParam, SystemInput, SystemMeta};

// -----------------------------------------------------------------------------
// SystemFunction

pub trait SystemFunction<Marker>: Send + Sync + 'static {
    type Param: SystemParam;
    type Input: SystemInput;
    type Output;

    fn run(
        &mut self,
        input: <Self::Input as SystemInput>::Inner<'_>,
        param: <Self::Param as SystemParam>::Item<'_, '_>,
    ) -> Self::Output;
}

macro_rules! impl_tuple {
    (0: []) => {
        impl<Out, Func> SystemFunction<fn() -> Out> for Func
        where
            Out: 'static,
            Func: Send + Sync + 'static,
            for <'a> &'a mut Func: FnMut() -> Out,
        {
            type Param = ();
            type Input = ();
            type Output = Out;
            
            fn run(&mut self, _input: (), _param: ()) -> Self::Output {
                #[inline(always)]
                fn call_func<Out>(mut f: impl FnMut() -> Out) -> Out {
                    f()
                }
                call_func(self)
            }
        }

        impl<In, Out, Func> SystemFunction<(In, fn() -> Out)> for Func
        where
            Out: 'static,
            In: SystemInput + 'static,
            Func: Send + Sync + 'static,
            for <'a> &'a mut Func: FnMut(In) -> Out,
            for <'a> &'a mut Func: FnMut(In::Param<'_>) -> Out,
        {
            type Param = ();
            type Input = In;
            type Output = Out;
            
            fn run(
                &mut self,
                input: In::Inner<'_>,
                _param: (),
            ) -> Out {
                #[inline(always)]
                fn call_func<In: SystemInput, Out>(
                    mut f: impl FnMut(In::Param<'_>) -> Out,
                    input: In::Param<'_>,
                ) -> Out {
                    f(input)
                }
                call_func(self, In::wrap(input))
            }
        }
    };
    (1 : [ $index:tt : $name:ident ]) => {
        #[cfg_attr(docsrs, doc(fake_variadic))]
        #[cfg_attr(docsrs, doc = "This trait is implemented for tuples up to 15 items long.")]
        #[allow(non_snake_case, reason = "macro generated implementation")]
        impl<Out, Func, $name: SystemParam> SystemFunction<fn($name) -> Out> for Func
        where
            Out: 'static,
            Func: Send + Sync + 'static,
            for <'a> &'a mut Func: FnMut( $name ) -> Out,
            for <'a> &'a mut Func: FnMut( <$name>::Item<'_, '_> ) -> Out,
        {
            type Param = ( $name, );
            type Input = ();
            type Output = Out;
            
            fn run(
                &mut self,
                _input: <Self::Input as SystemInput>::Inner<'_>,
                param: <Self::Param as SystemParam>::Item<'_, '_>,
            ) -> Self::Output {
                #[inline(always)]
                fn call_func<Out, $name>(
                    mut f: impl FnMut($name)->Out,
                    $name: $name,
                )->Out{
                    f($name)
                }
                let ($name,) = param;
                call_func(self, $name)
            }
        }

        #[cfg_attr(docsrs, doc(fake_variadic))]
        #[cfg_attr(docsrs, doc = "This trait is implemented for tuples up to 15 items long.")]
        #[allow(non_snake_case, reason = "macro generated implementation")]
        impl<In, Out, Func, $name: SystemParam> SystemFunction<(In, fn($name) -> Out)> for Func
        where
            Out: 'static,
            In: SystemInput + 'static,
            Func: Send + Sync + 'static,
            for <'a> &'a mut Func: FnMut( In, $name ) -> Out,
            for <'a> &'a mut Func: FnMut( In::Param<'_>, <$name>::Item<'_, '_> ) -> Out,
        {
            type Param = ( $name, );
            type Input = In;
            type Output = Out;
            
            fn run(
                &mut self,
                input: <Self::Input as SystemInput>::Inner<'_>,
                param: <Self::Param as SystemParam>::Item<'_, '_>,
            ) -> Self::Output {
                #[inline(always)]
                fn call_func<In: SystemInput, Out, $name>(
                    mut f: impl FnMut(In::Param<'_>, $name)->Out,
                    input: In::Param<'_>,
                    $name: $name,
                )->Out{
                    f(input, $name)
                }
                let ($name,) = param;
                call_func(self, In::wrap(input), $name)
            }
        }
    };
    ($num:literal : [$($index:tt : $name:ident),*]) => {
        #[cfg_attr(docsrs, doc(hidden))]
        #[allow(non_snake_case, reason = "macro generated implementation")]
        impl<Out, Func, $($name: SystemParam),*> SystemFunction<fn($($name),*) -> Out> for Func
        where
            Out: 'static,
            Func: Send + Sync + 'static,
            for <'a> &'a mut Func: FnMut( $($name),* ) -> Out,
            for <'a> &'a mut Func: FnMut( $(<$name>::Item<'_, '_>),* ) -> Out,
        {
            type Param = ( $($name),* );
            type Input = ();
            type Output = Out;
            
            fn run(
                &mut self,
                _input: <Self::Input as SystemInput>::Inner<'_>,
                param: <Self::Param as SystemParam>::Item<'_, '_>,
            ) -> Self::Output {
                #[inline(always)]
                fn call_func<Out, $($name),*>(
                    mut f: impl FnMut($($name),*)->Out,
                    $($name: $name,)*
                )->Out{
                    f($($name,)*)
                }
                let ( $($name),* ) = param;
                call_func(self, $($name,)*)
            }
        }

        #[cfg_attr(docsrs, doc(hidden))]
        #[allow(non_snake_case, reason = "macro generated implementation")]
        impl<In, Out, Func, $($name: SystemParam),*> SystemFunction<(In, fn($($name),*) -> Out)> for Func
        where
            Out: 'static,
            In: SystemInput + 'static,
            Func: Send + Sync + 'static,
            for <'a> &'a mut Func: FnMut( In, $($name),* ) -> Out,
            for <'a> &'a mut Func: FnMut( In::Param<'_>, $(<$name>::Item<'_, '_>),* ) -> Out,
        {
            type Param = ( $($name),* );
            type Input = In;
            type Output = Out;
            
            fn run(
                &mut self,
                input: <Self::Input as SystemInput>::Inner<'_>,
                param: <Self::Param as SystemParam>::Item<'_, '_>,
            ) -> Self::Output {
                #[inline(always)]
                fn call_func<In: SystemInput, Out, $($name),*>(
                    mut f: impl FnMut(In::Param<'_>, $($name),*) -> Out,
                    input: In::Param<'_>,
                    $($name: $name,)*
                )->Out{
                    f(input, $($name,)*)
                }
                let ( $($name),* ) = param;
                call_func(self, In::wrap(input), $($name,)*)
            }
        }
    }
}

vc_utils::range_invoke!(impl_tuple, 15: P);

// -----------------------------------------------------------------------------
// FunctionSystem

struct FunctionSystemState<P: SystemParam> {
    param: P::State,
    world_id: WorldId,
}

pub struct FunctionSystem<Marker, In, Out, F>
where
    F: SystemFunction<Marker>,
{
    func: F,
    state: Option<FunctionSystemState<F::Param>>,
    meta: SystemMeta,
    // NOTE: PhantomData<fn()-> T> gives this safe Send/Sync impls
    _marker: PhantomData<fn(In) -> (Marker, Out)>,
}

impl<Marker, In, Out, F> FunctionSystem<Marker, In, Out, F>
where
    F: SystemFunction<Marker>,
{
    #[inline]
    fn new(func: F, meta: SystemMeta, state: Option<FunctionSystemState<F::Param>>) -> Self {
        Self {
            func,
            state,
            meta,
            _marker: PhantomData,
        }
    }
}

impl<Marker, In, Out, F> Clone for FunctionSystem<Marker, In, Out, F>
where
    F: SystemFunction<Marker> + Clone,
{
    fn clone(&self) -> Self {
        Self {
            func: self.func.clone(),
            state: None,
            meta: SystemMeta::new::<F>(),
            _marker: PhantomData,
        }
    }
}


