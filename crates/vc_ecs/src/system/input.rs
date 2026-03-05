pub trait SystemInput: Sized {
    type Inner<'i>;
    type Param<'i>: SystemInput;

    fn wrap(this: Self::Inner<'_>) -> Self::Param<'_>;
}

#[derive(Debug)]
#[repr(transparent)]
pub struct In<T>(pub T);

impl<T: 'static> SystemInput for In<T> {
    type Inner<'i> = T;
    type Param<'i> = In<T>;

    #[inline(always)]
    fn wrap(this: Self::Inner<'_>) -> Self::Param<'_> {
        In(this)
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct InRef<'i, T: ?Sized>(pub &'i T);

impl<T: ?Sized + 'static> SystemInput for InRef<'_, T> {
    type Inner<'i> = &'i T;
    type Param<'i> = InRef<'i, T>;

    #[inline(always)]
    fn wrap(this: Self::Inner<'_>) -> Self::Param<'_> {
        InRef(this)
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct InMut<'a, T: ?Sized>(pub &'a mut T);

impl<T: ?Sized + 'static> SystemInput for InMut<'_, T> {
    type Inner<'i> = &'i mut T;
    type Param<'i> = InMut<'i, T>;

    #[inline(always)]
    fn wrap(this: Self::Inner<'_>) -> Self::Param<'_> {
        InMut(this)
    }
}

macro_rules! impl_tuple {
    (0: []) => {
        impl SystemInput for () {
            type Inner<'i> = ();
            type Param<'i> = ();

            #[inline(always)]
            fn wrap(_: Self::Inner<'_>) -> Self::Param<'_> {}
        }
    };
    (1 : [ $index:tt : $name:ident ]) => {
        #[cfg_attr(docsrs, doc(fake_variadic))]
        #[cfg_attr(docsrs, doc = "This trait is implemented for tuples up to 8 items long.")]
        impl<$name: SystemInput> SystemInput for ($name,) {
            type Inner<'i> = ( <$name>::Inner<'i>, );
            type Param<'i> = ( <$name>::Param<'i>, );

            fn wrap(this: Self::Inner<'_>) -> Self::Param<'_> {
                ( <$name as SystemInput>::wrap(this.0), )
            }
        }
    };
    ($num:literal : [$($index:tt : $name:ident),*]) => {
        #[cfg_attr(docsrs, doc(hidden))]
        impl<$($name: SystemInput),*> SystemInput for ($($name),*) {
            type Inner<'i> = ( $( <$name>::Inner<'i> ),* );
            type Param<'i> = ( $( <$name>::Param<'i> ),* );

            fn wrap(this: Self::Inner<'_>) -> Self::Param<'_> {
                ( $( <$name as SystemInput>::wrap(this.$index) ),* )
            }
        }
    };
}

vc_utils::range_invoke!(impl_tuple, 8: P);
