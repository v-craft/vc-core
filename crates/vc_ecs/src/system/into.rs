use crate::system::{System, SystemInput};



pub trait IntoSystem<In: SystemInput, Out>: Sized {
    type System: System<In = In, Out = Out>;

    fn into_system(this: Self) -> Self::System;

    fn pipe<B, BIn, BOut>(self, system: B) -> IntoPipeSystem<Self, B>
    where
        Out: 'static,
        B: IntoSystem<BIn, BOut>,
        for<'a> BIn: SystemInput<Inner<'a> = Out>,
    {
        IntoPipeSystem { a: self, b: system }
    }

    fn map<T, F>(self, f: F) -> IntoAdapterSystem<Self, F>
    where
        F: Send + Sync + 'static + FnMut(Out) -> T,
    {
        IntoAdapterSystem { system: self, func: f }
    }

}

/// An [`IntoSystem`] creating an instance of [`PipeSystem`].
#[derive(Clone)]
pub struct IntoPipeSystem<A, B> {
    a: A,
    b: B,
}

#[derive(Clone)]
pub struct IntoAdapterSystem<S, F> {
    system: S,
    func: F,
}

