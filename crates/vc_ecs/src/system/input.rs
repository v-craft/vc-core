pub trait SystemInput: Sized {
    type Data<'i>;
    type Item<'i>: SystemInput;

    fn wrap(this: Self::Data<'_>) -> Self::Item<'_>;
}

#[derive(Debug)]
#[repr(transparent)]
pub struct SystemIn<T: SystemInput>(pub T);

#[derive(Debug)]
#[repr(transparent)]
pub struct In<T>(pub T);

impl<T: 'static> SystemInput for In<T> {
    type Data<'i> = T;
    type Item<'i> = In<T>;

    #[inline(always)]
    fn wrap(this: Self::Data<'_>) -> Self::Item<'_> {
        In(this)
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct InRef<'i, T: ?Sized>(pub &'i T);

impl<T: ?Sized + 'static> SystemInput for InRef<'_, T> {
    type Data<'i> = &'i T;
    type Item<'i> = InRef<'i, T>;

    #[inline(always)]
    fn wrap(this: Self::Data<'_>) -> Self::Item<'_> {
        InRef(this)
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct InMut<'a, T: ?Sized>(pub &'a mut T);

impl<T: ?Sized + 'static> SystemInput for InMut<'_, T> {
    type Data<'i> = &'i mut T;
    type Item<'i> = InMut<'i, T>;

    #[inline(always)]
    fn wrap(this: Self::Data<'_>) -> Self::Item<'_> {
        InMut(this)
    }
}

macro_rules! impl_tuple {
    (0: []) => {
        impl SystemInput for () {
            type Data<'i> = ();
            type Item<'i> = ();

            #[inline(always)]
            fn wrap(_: Self::Data<'_>) -> Self::Item<'_> {}
        }
    };
    (1 : [ $index:tt : $name:ident ]) => {
        #[cfg_attr(docsrs, doc(fake_variadic))]
        #[cfg_attr(docsrs, doc = "This trait is implemented for tuples up to 8 items long.")]
        impl<$name: SystemInput> SystemInput for ($name,) {
            type Data<'i> = ( <$name>::Data<'i>, );
            type Item<'i> = ( <$name>::Item<'i>, );

            fn wrap(this: Self::Data<'_>) -> Self::Item<'_> {
                ( <$name as SystemInput>::wrap(this.0), )
            }
        }
    };
    ($num:literal : [$($index:tt : $name:ident),*]) => {
        #[cfg_attr(docsrs, doc(hidden))]
        impl<$($name: SystemInput),*> SystemInput for ($($name),*) {
            type Data<'i> = ( $( <$name>::Data<'i> ),* );
            type Item<'i> = ( $( <$name>::Item<'i> ),* );

            fn wrap(this: Self::Data<'_>) -> Self::Item<'_> {
                ( $( <$name as SystemInput>::wrap(this.$index) ),* )
            }
        }
    };
}

vc_utils::range_invoke!(impl_tuple, 12);
