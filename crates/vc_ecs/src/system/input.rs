use core::ops::{Deref, DerefMut};

use vc_utils::range_invoke;

pub trait SystemInput: Sized {
    type Param<'i>: SystemInput;
    type Inner<'i>;

    fn wrap(this: Self::Inner<'_>) -> Self::Param<'_>;
}

// -----------------------------------------------------------------------------
// In

#[derive(Debug)]
pub struct In<T>(pub T);

impl<T: 'static> SystemInput for In<T> {
    type Param<'i> = In<T>;
    type Inner<'i> = T;

    fn wrap(this: Self::Inner<'_>) -> Self::Param<'_> {
        In(this)
    }
}

impl<T> Deref for In<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for In<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

// -----------------------------------------------------------------------------
// InRef

#[derive(Debug)]
pub struct InRef<'a, T: ?Sized>(pub &'a T);

impl<T: ?Sized + 'static> SystemInput for InRef<'_, T> {
    type Param<'a> = InRef<'a, T>;
    type Inner<'a> = &'a T;

    fn wrap(this: Self::Inner<'_>) -> Self::Param<'_> {
        InRef(this)
    }
}

impl<'a, T: ?Sized> Deref for InRef<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.0
    }
}

// -----------------------------------------------------------------------------
// InRef

#[derive(Debug)]
pub struct InMut<'a, T: ?Sized>(pub &'a mut T);

impl<T: ?Sized + 'static> SystemInput for InMut<'_, T> {
    type Param<'i> = InMut<'i, T>;
    type Inner<'i> = &'i mut T;

    fn wrap(this: Self::Inner<'_>) -> Self::Param<'_> {
        InMut(this)
    }
}

impl<'i, T: ?Sized> Deref for InMut<'i, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<'i, T: ?Sized> DerefMut for InMut<'i, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0
    }
}

// -----------------------------------------------------------------------------
// Tuple

macro_rules! impl_system_input_tuple {
    (0: []) => {
        impl SystemInput for () {
            type Param<'i> = ();
            type Inner<'i> = ();

            fn wrap(this: Self::Inner<'_>) -> Self::Param<'_> { this }
        }
    };
    (1 : [ $index:tt : $name:ident ]) => {
        #[cfg_attr(docsrs, doc(fake_variadic))]
        #[cfg_attr(docsrs, doc = "This trait is implemented for tuples up to 15 items long.")]
        impl<$name: SystemInput> SystemInput for ($name,) {
            type Param<'i> = (<$name>::Param<'i>,);
            type Inner<'i> = (<$name>::Inner<'i>,);

            #[allow(non_snake_case, reason = "macro implementation.")]
            fn wrap(this: Self::Inner<'_>) -> Self::Param<'_> {
                let ($name,) = this;
                ( $name::wrap($name), )
            }
        }
    };
    ($num:literal : [$($index:tt : $name:ident),*]) => {
        #[cfg_attr(docsrs, doc(hidden))]
        impl<$($name: SystemInput),*> SystemInput for ($($name,)*) {
            type Param<'i> = ($($name::Param<'i>,)*);
            type Inner<'i> = ($($name::Inner<'i>,)*);

            #[allow(non_snake_case, reason = "macro implementation.")]
            fn wrap(this: Self::Inner<'_>) -> Self::Param<'_> {
                let ($($name,)*) = this;
                ($($name::wrap($name),)*)
            }
        }
    };
}

range_invoke! {
    impl_system_input_tuple, 8: P
}
