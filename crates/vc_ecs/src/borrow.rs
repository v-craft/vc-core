use core::iter::FusedIterator;
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};
use core::ptr::NonNull;

use vc_ptr::{Ptr, PtrMut};

use crate::change_detection::DetectChanges;
use crate::component::{ComponentTicksMut, ComponentTicksRef, NonSendResource, Resource};
use crate::component::{ComponentTicksSliceMut, ComponentTicksSliceRef};
use crate::tick::Tick;

// -----------------------------------------------------------------------------
// Res

pub struct Res<'w, T: Resource> {
    pub(crate) value: &'w T,
    pub(crate) ticks: ComponentTicksRef<'w>,
}

// -----------------------------------------------------------------------------
// ResMut

pub struct ResMut<'w, T: Resource> {
    pub(crate) value: &'w mut T,
    pub(crate) ticks: ComponentTicksMut<'w>,
}

// -----------------------------------------------------------------------------
// NonSend

pub struct NonSend<'w, T: NonSendResource> {
    pub(crate) value: &'w T,
    pub(crate) ticks: ComponentTicksRef<'w>,
}

// -----------------------------------------------------------------------------
// NonSendMut

pub struct NonSendMut<'w, T: NonSendResource> {
    pub(crate) value: &'w mut T,
    pub(crate) ticks: ComponentTicksMut<'w>,
}

// -----------------------------------------------------------------------------
// Ref

pub struct Ref<'w, T: ?Sized> {
    pub(crate) value: &'w T,
    pub(crate) ticks: ComponentTicksRef<'w>,
}

// -----------------------------------------------------------------------------
// Mut

pub struct Mut<'w, T: ?Sized> {
    pub(crate) value: &'w mut T,
    pub(crate) ticks: ComponentTicksMut<'w>,
}

// -----------------------------------------------------------------------------
// SliceRef

pub struct SliceRef<'w, T> {
    pub(crate) value: &'w [T],
    pub(crate) ticks: ComponentTicksSliceRef<'w>,
}

// -----------------------------------------------------------------------------
// SliceRef

pub struct SliceMut<'w, T> {
    pub(crate) value: &'w mut [T],
    pub(crate) ticks: ComponentTicksSliceMut<'w>,
}

// -----------------------------------------------------------------------------
// Untyped

pub struct UntypedRef<'w> {
    pub(crate) value: Ptr<'w>,
    pub(crate) ticks: ComponentTicksRef<'w>,
}

// -----------------------------------------------------------------------------
// MutUntyped

pub struct UntypedMut<'w> {
    pub(crate) value: PtrMut<'w>,
    pub(crate) ticks: ComponentTicksMut<'w>,
}

// -----------------------------------------------------------------------------
// UntypedSliceRef

pub struct UntypedSliceRef<'w> {
    pub(crate) value: Ptr<'w>,
    pub(crate) ticks: ComponentTicksSliceRef<'w>,
}

// -----------------------------------------------------------------------------
// UntypedSliceMut

pub struct UntypedSliceMut<'w> {
    pub(crate) value: PtrMut<'w>,
    pub(crate) ticks: ComponentTicksSliceMut<'w>,
}

// -----------------------------------------------------------------------------
// From

impl<'w, T: Resource> From<ResMut<'w, T>> for Mut<'w, T> {
    #[inline]
    fn from(other: ResMut<'w, T>) -> Mut<'w, T> {
        Mut {
            value: other.value,
            ticks: other.ticks,
        }
    }
}

impl<'w, T: Resource> From<ResMut<'w, T>> for Res<'w, T> {
    #[inline]
    fn from(other: ResMut<'w, T>) -> Self {
        Self {
            value: other.value,
            ticks: other.ticks.into(),
        }
    }
}

impl<'w, T: Resource> From<Res<'w, T>> for Ref<'w, T> {
    #[inline]
    fn from(other: Res<'w, T>) -> Self {
        Self {
            value: other.value,
            ticks: other.ticks,
        }
    }
}

impl<'w, T: NonSendResource> From<NonSendMut<'w, T>> for Mut<'w, T> {
    #[inline]
    fn from(other: NonSendMut<'w, T>) -> Mut<'w, T> {
        Mut {
            value: other.value,
            ticks: other.ticks,
        }
    }
}

impl<'w, T: NonSendResource> From<NonSendMut<'w, T>> for NonSend<'w, T> {
    #[inline]
    fn from(other: NonSendMut<'w, T>) -> Self {
        Self {
            value: other.value,
            ticks: other.ticks.into(),
        }
    }
}

impl<'w, T: NonSendResource> From<NonSend<'w, T>> for Ref<'w, T> {
    #[inline]
    fn from(other: NonSend<'w, T>) -> Ref<'w, T> {
        Ref {
            value: other.value,
            ticks: other.ticks,
        }
    }
}

impl<'w, T: ?Sized> From<Ref<'w, T>> for UntypedRef<'w> {
    #[inline]
    fn from(other: Ref<'w, T>) -> Self {
        UntypedRef {
            value: other.value.into(),
            ticks: other.ticks,
        }
    }
}

impl<'w, T: ?Sized> From<Mut<'w, T>> for Ref<'w, T> {
    #[inline]
    fn from(other: Mut<'w, T>) -> Self {
        Self {
            value: other.value,
            ticks: other.ticks.into(),
        }
    }
}

impl<'w, T: ?Sized> From<Mut<'w, T>> for UntypedMut<'w> {
    #[inline]
    fn from(other: Mut<'w, T>) -> Self {
        UntypedMut {
            value: other.value.into(),
            ticks: other.ticks,
        }
    }
}

impl<'w, T> From<SliceMut<'w, T>> for SliceRef<'w, T> {
    #[inline]
    fn from(other: SliceMut<'w, T>) -> Self {
        SliceRef {
            value: other.value,
            ticks: other.ticks.into(),
        }
    }
}

impl<'w> From<UntypedMut<'w>> for UntypedRef<'w> {
    #[inline]
    fn from(other: UntypedMut<'w>) -> Self {
        UntypedRef {
            value: other.value.into(),
            ticks: other.ticks.into(),
        }
    }
}

impl<'w> From<UntypedSliceMut<'w>> for UntypedSliceRef<'w> {
    #[inline]
    fn from(other: UntypedSliceMut<'w>) -> Self {
        UntypedSliceRef {
            value: other.value.into(),
            ticks: other.ticks.into(),
        }
    }
}

// -----------------------------------------------------------------------------
// IntoIterator

impl<'w, 'a, T: Resource> IntoIterator for &'a mut ResMut<'w, T>
where
    &'a mut T: IntoIterator,
{
    type Item = <&'a mut T as IntoIterator>::Item;
    type IntoIter = <&'a mut T as IntoIterator>::IntoIter;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        *self.ticks.changed = self.ticks.this_run;
        self.value.into_iter()
    }
}

impl<'w, 'a, T: Resource> IntoIterator for &'a ResMut<'w, T>
where
    &'a T: IntoIterator,
{
    type Item = <&'a T as IntoIterator>::Item;
    type IntoIter = <&'a T as IntoIterator>::IntoIter;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.value.into_iter()
    }
}

impl<'w, 'a, T: Resource> IntoIterator for &'a Res<'w, T>
where
    &'a T: IntoIterator,
{
    type Item = <&'a T as IntoIterator>::Item;
    type IntoIter = <&'a T as IntoIterator>::IntoIter;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.value.into_iter()
    }
}

impl<'w, 'a, T: NonSendResource> IntoIterator for &'a mut NonSendMut<'w, T>
where
    &'a mut T: IntoIterator,
{
    type Item = <&'a mut T as IntoIterator>::Item;
    type IntoIter = <&'a mut T as IntoIterator>::IntoIter;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        *self.ticks.changed = self.ticks.this_run;
        self.value.into_iter()
    }
}

impl<'w, 'a, T: NonSendResource> IntoIterator for &'a NonSendMut<'w, T>
where
    &'a T: IntoIterator,
{
    type Item = <&'a T as IntoIterator>::Item;
    type IntoIter = <&'a T as IntoIterator>::IntoIter;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.value.into_iter()
    }
}

impl<'w, 'a, T: NonSendResource> IntoIterator for &'a NonSend<'w, T>
where
    &'a T: IntoIterator,
{
    type Item = <&'a T as IntoIterator>::Item;
    type IntoIter = <&'a T as IntoIterator>::IntoIter;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.value.into_iter()
    }
}

impl<'w, 'a, T> IntoIterator for &'a mut Mut<'w, T>
where
    &'a mut T: IntoIterator,
{
    type Item = <&'a mut T as IntoIterator>::Item;
    type IntoIter = <&'a mut T as IntoIterator>::IntoIter;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        *self.ticks.changed = self.ticks.this_run;
        self.value.into_iter()
    }
}

impl<'w, 'a, T> IntoIterator for &'a Mut<'w, T>
where
    &'a T: IntoIterator,
{
    type Item = <&'a T as IntoIterator>::Item;
    type IntoIter = <&'a T as IntoIterator>::IntoIter;
    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.value.into_iter()
    }
}

impl<'w, 'a, T> IntoIterator for &'a Ref<'w, T>
where
    &'a T: IntoIterator,
{
    type Item = <&'a T as IntoIterator>::Item;
    type IntoIter = <&'a T as IntoIterator>::IntoIter;
    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.value.into_iter()
    }
}

// -----------------------------------------------------------------------------
// impl_debug

macro_rules! impl_debug {
    ($name:ident < $( $generics:tt ),+ > $($traits:ident)?) => {
        impl<$($generics),* : ?Sized $(+ $traits)?> ::core::fmt::Debug for $name<$($generics),*>
            where T: ::core::fmt::Debug
        {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_tuple(stringify!($name))
                    .field(&self.value)
                    .finish()
            }
        }
    };
}

impl_debug!(ResMut<'w, T> Resource);
impl_debug!(Res<'w, T> Resource);
impl_debug!(NonSendMut<'w, T> NonSendResource);
impl_debug!(NonSend<'w, T> NonSendResource);
impl_debug!(Mut<'w, T>);
impl_debug!(Ref<'w, T>);

// -----------------------------------------------------------------------------
// impl_ref_methods

macro_rules! impl_ref_methods {
    ($name:ident < $( $generics:tt ),+ >, $target:ty, $($traits:ident)?) => {
        impl<$($generics),* : ?Sized $(+ $traits)?> $name<$($generics),*> {
            /// Consumes self and returns the inner reference `&T` with the same lifetime.
            #[inline(always)]
            pub fn into_inner(self) -> &'w $target {
                self.value
            }

            /// Creates a copy with the same lifetime.
            ///
            /// Since this is a shared reference, the original and copy do not interfere.
            #[inline]
            pub fn reborrow(&self) -> Self {
                Self {
                    value: self.value,
                    ticks: self.ticks.clone(),
                }
            }

            /// Transforms the reference type via a function, preserving the lifetime.
            ///
            /// Returns the generic [`Ref`] container.
            #[inline(always)]
            pub fn map_type<U: ?Sized>(
                self,
                f: impl FnOnce(&$target) -> &U,
            ) -> Ref<'w, U> {
                Ref {
                    value: f(self.value),
                    ticks: self.ticks,
                }
            }

            /// Transforms the reference type via a function, preserving the lifetime.
            ///
            /// Returns the generic [`Ref`] container, or an error if the transformation fails.
            #[inline]
            pub fn try_map_type<U: ?Sized, E>(
                self,
                f: impl FnOnce(&$target) -> Result<&U, E>,
            ) -> Result<Ref<'w, U>, E> {
                let value = f(self.value);
                value.map(|value| Ref {
                    value,
                    ticks: self.ticks,
                })
            }

            /// Dereferences the inner type, e.g., converts `Ref<'a, Box<T>>` to `Ref<'a, T>`.
            ///
            /// Returns the generic [`Ref`] container.
            #[inline]
            pub fn into_deref(self) -> Ref<'w, <$target as ::core::ops::Deref>::Target>
                where $target: ::core::ops::Deref
            {
                self.map_type(|v| v.deref())
            }
        }
    };
}

impl_ref_methods!(Res<'w, T>, T, Resource);
impl_ref_methods!(NonSend<'w, T>, T, NonSendResource);
impl_ref_methods!(Ref<'w, T>, T,);

// -----------------------------------------------------------------------------
// impl_mut_methods

macro_rules! impl_mut_methods {
    ($name:ident < $( $generics:tt ),+ >, $target:ty, $($traits:ident)?) => {
        impl<$($generics),* : ?Sized $(+ $traits)?> $name<$($generics),*> {
            /// Consumes self and returns the inner reference `&mut T` with the
            /// same lifetime, marking the target as changed.
            #[inline]
            pub fn into_inner(self) -> &'w mut $target {
                *self.ticks.changed = self.ticks.this_run;
                self.value
            }

            /// Returns a shorter-lived version of self, with borrow checker guarantees.
            ///
            /// This function does not mark the target as changed.
            pub fn reborrow(&mut self) -> $name<'_, $target> {
                $name {
                    value: self.value,
                    ticks: ComponentTicksMut {
                        added: self.ticks.added,
                        changed: self.ticks.changed,
                        last_run: self.ticks.last_run,
                        this_run: self.ticks.this_run,
                    },
                }
            }

            /// Transforms the reference type via a function, preserving the lifetime.
            ///
            /// Returns the generic [`Mut`] container.
            ///
            /// This function is assumed to only change the type, not modify data.
            /// Modifying data through the mutable reference in the closure is undefined behavior
            /// (data may be modified without triggering change events).
            #[inline(always)]
            pub fn map_type<U: ?Sized>(
                self,
                f: impl FnOnce(&mut $target) -> &mut U,
            ) -> Mut<'w, U> {
                Mut {
                    value: f(self.value),
                    ticks: self.ticks,
                }
            }

            /// Transforms the reference type via a function, preserving the lifetime.
            ///
            /// Returns the generic [`Mut`] container, or an error if the transformation fails.
            ///
            /// This function is assumed to only change the type, not modify data.
            /// Modifying data through the mutable reference in the closure is undefined behavior
            /// (data may be modified without triggering change events).
            #[inline]
            pub fn try_map_type<U: ?Sized, E>(
                self,
                f: impl FnOnce(&mut $target) -> Result<&mut U, E>,
            ) -> Result<Mut<'w, U>, E> {
                let value = f(self.value);
                value.map(|value| Mut {
                    value,
                    ticks: self.ticks,
                })
            }

            /// Dereferences the inner type, e.g., converts `Mut<'a, Box<T>>` to `Mut<'a, T>`.
            ///
            /// Returns the generic [`Mut`] container.
            ///
            /// This function does not set the change flag.
            #[inline]
            pub fn into_deref(self) -> Mut<'w, <$target as ::core::ops::Deref>::Target>
                where $target: ::core::ops::DerefMut
            {
                self.map_type(|v| v.deref_mut())
            }
        }
    };
}

impl_mut_methods!(ResMut<'w, T>, T, Resource);
impl_mut_methods!(NonSendMut<'w, T>, T, NonSendResource);
impl_mut_methods!(Mut<'w, T>, T,);

// -----------------------------------------------------------------------------
// impl_change_detection_and_deref

macro_rules! impl_change_detection_and_deref {
    ($name:ident < $( $generics:tt ),+ >, $target:ty, $($traits:ident)?) => {
        impl<$($generics),* : ?Sized $(+ $traits)?> DetectChanges for $name<$($generics),*> {
            #[inline]
            fn is_added(&self) -> bool {
                self.ticks
                    .added
                    .is_newer_than(self.ticks.last_run, self.ticks.this_run)
            }

            #[inline]
            fn is_changed(&self) -> bool {
                self.ticks
                    .changed
                    .is_newer_than(self.ticks.last_run, self.ticks.this_run)
            }

            #[inline(always)]
            fn changed_tick(&self) -> Tick {
                *self.ticks.changed
            }

            #[inline(always)]
            fn added_tick(&self) -> Tick {
                *self.ticks.added
            }

        }

        impl<$($generics),*: ?Sized $(+ $traits)?> ::core::ops::Deref for $name<$($generics),*> {
            type Target = $target;

            #[inline(always)]
            fn deref(&self) -> &Self::Target {
                self.value
            }
        }

        impl<$($generics),* $(: $traits)?> AsRef<$target> for $name<$($generics),*> {
            #[inline(always)]
            fn as_ref(&self) -> &$target {
                self.value
            }
        }
    }
}

impl_change_detection_and_deref!(Res<'w, T>, T, Resource);
impl_change_detection_and_deref!(ResMut<'w, T>, T, Resource);
impl_change_detection_and_deref!(NonSend<'w, T>, T, NonSendResource);
impl_change_detection_and_deref!(NonSendMut<'w, T>, T, NonSendResource);
impl_change_detection_and_deref!(Ref<'w, T>, T,);
impl_change_detection_and_deref!(Mut<'w, T>, T,);

// -----------------------------------------------------------------------------
// SliceRef

impl<T: core::fmt::Debug> core::fmt::Debug for SliceRef<'_, T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("SliceRef").field(&self.value).finish()
    }
}

impl<'w, T> SliceRef<'w, T> {
    /// Consumes self and returns the inner reference `&T` with the same lifetime.
    #[inline(always)]
    pub fn into_inner(self) -> &'w [T] {
        self.value
    }

    /// Creates a copy with the **same** lifetime.
    ///
    /// Since this is a shared reference, the original and copy do not interfere.
    #[inline(always)]
    pub fn reborrow(&self) -> SliceRef<'w, T> {
        Self {
            value: self.value,
            ticks: self.ticks.clone(),
        }
    }
}

impl<'w, T> Deref for SliceRef<'w, T> {
    type Target = [T];

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        self.value
    }
}

impl<'w, T> AsRef<[T]> for SliceRef<'w, T> {
    #[inline(always)]
    fn as_ref(&self) -> &[T] {
        self.value
    }
}

impl<'w, T> IntoIterator for SliceRef<'w, T> {
    type Item = Ref<'w, T>;
    type IntoIter = SliceRefIterator<'w, T>;

    fn into_iter(self) -> Self::IntoIter {
        unsafe {
            SliceRefIterator {
                len: self.value.len(),
                value: NonNull::new_unchecked(self.value.as_ptr().cast_mut()),
                added: NonNull::new_unchecked(self.ticks.added.as_ptr().cast_mut()),
                changed: NonNull::new_unchecked(self.ticks.changed.as_ptr().cast_mut()),
                last_run: self.ticks.last_run,
                this_run: self.ticks.this_run,
                _marker: PhantomData,
            }
        }
    }
}

pub struct SliceRefIterator<'w, T> {
    len: usize,
    value: NonNull<T>,
    added: NonNull<Tick>,
    changed: NonNull<Tick>,
    last_run: Tick,
    this_run: Tick,
    _marker: PhantomData<&'w [T]>,
}

impl<'w, T> Iterator for SliceRefIterator<'w, T> {
    type Item = Ref<'w, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            return None;
        }

        unsafe {
            let ret: Ref<'w, T> = Ref {
                value: self.value.as_ref(),
                ticks: ComponentTicksRef {
                    added: self.added.as_ref(),
                    changed: self.changed.as_ref(),
                    last_run: self.last_run,
                    this_run: self.this_run,
                },
            };

            self.value = self.value.add(1);
            self.added = self.added.add(1);
            self.changed = self.changed.add(1);
            self.len -= 1;

            Some(ret)
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<T> ExactSizeIterator for SliceRefIterator<'_, T> {}
impl<T> FusedIterator for SliceRefIterator<'_, T> {}

// -----------------------------------------------------------------------------
// SliceMut

impl<T: core::fmt::Debug> core::fmt::Debug for SliceMut<'_, T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("SliceMut").field(&self.value).finish()
    }
}

impl<'w, T> SliceMut<'w, T> {
    fn mark_all_changed(&mut self) {
        self.ticks
            .changed
            .iter_mut()
            .for_each(|it| *it = self.ticks.this_run);
    }

    /// Consumes self and returns the inner reference `&T` with the same lifetime.
    #[inline]
    pub fn into_inner(mut self) -> &'w mut [T] {
        self.mark_all_changed();
        self.value
    }

    /// Returns a shorter-lived version of self, with borrow checker guarantees.
    ///
    /// This function does not mark the target as changed.
    #[inline]
    pub fn reborrow(&mut self) -> SliceMut<'_, T> {
        SliceMut {
            value: self.value,
            ticks: ComponentTicksSliceMut {
                added: self.ticks.added,
                changed: self.ticks.changed,
                last_run: self.ticks.last_run,
                this_run: self.ticks.this_run,
            },
        }
    }
}

impl<'w, T> Deref for SliceMut<'w, T> {
    type Target = [T];

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        self.value
    }
}

impl<'w, T> DerefMut for SliceMut<'w, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.mark_all_changed();
        self.value
    }
}

impl<'w, T> AsRef<[T]> for SliceMut<'w, T> {
    #[inline(always)]
    fn as_ref(&self) -> &[T] {
        self.value
    }
}

impl<'w, T> AsMut<[T]> for SliceMut<'w, T> {
    #[inline]
    fn as_mut(&mut self) -> &mut [T] {
        self.mark_all_changed();
        self.value
    }
}

impl<'w, T> IntoIterator for SliceMut<'w, T> {
    type Item = Mut<'w, T>;
    type IntoIter = SliceMutIterator<'w, T>;
    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        unsafe {
            SliceMutIterator {
                len: self.value.len(),
                value: NonNull::new_unchecked(self.value.as_ptr().cast_mut()),
                added: NonNull::new_unchecked(self.ticks.added.as_ptr().cast_mut()),
                changed: NonNull::new_unchecked(self.ticks.changed.as_ptr().cast_mut()),
                last_run: self.ticks.last_run,
                this_run: self.ticks.this_run,
                _marker: PhantomData,
            }
        }
    }
}

pub struct SliceMutIterator<'w, T> {
    len: usize,
    value: NonNull<T>,
    added: NonNull<Tick>,
    changed: NonNull<Tick>,
    last_run: Tick,
    this_run: Tick,
    _marker: PhantomData<&'w [T]>,
}

impl<'w, T> Iterator for SliceMutIterator<'w, T> {
    type Item = Mut<'w, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            return None;
        }

        unsafe {
            let ret: Mut<'w, T> = Mut {
                value: self.value.as_mut(),
                ticks: ComponentTicksMut {
                    added: self.added.as_mut(),
                    changed: self.changed.as_mut(),
                    last_run: self.last_run,
                    this_run: self.this_run,
                },
            };

            self.value = self.value.add(1);
            self.added = self.added.add(1);
            self.changed = self.changed.add(1);
            self.len -= 1;

            Some(ret)
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<T> ExactSizeIterator for SliceMutIterator<'_, T> {}
impl<T> FusedIterator for SliceMutIterator<'_, T> {}

// -----------------------------------------------------------------------------
// UntypedRef : Method Implementation

impl<'w> UntypedRef<'w> {
    /// Consumes self and returns the inner [`PtrMut`].
    #[inline(always)]
    pub fn into_inner(self) -> Ptr<'w> {
        self.value
    }

    /// Creates a copy with the same lifetime.
    ///
    /// Since this is a shared reference, the original and copy do not interfere.
    #[inline(always)]
    pub fn reborrow(&self) -> UntypedRef<'w> {
        Self {
            value: self.value,
            ticks: self.ticks.clone(),
        }
    }

    /// Checks whether this value has changed since the given tick.
    #[inline]
    pub fn has_changed_since(&self, tick: Tick) -> bool {
        self.ticks.changed.is_newer_than(tick, self.ticks.this_run)
    }

    /// Converts self to a [`Ref`] by specifying the reference type via a function.
    ///
    /// Consider using [`with_type`](Self::with_type) instead for `Sized` types without
    /// complex operations.
    ///
    /// # Safety
    /// `T` must be the erased pointee type for this [`UntypedRef`].
    #[inline(always)]
    pub unsafe fn map_type<T: ?Sized>(self, f: impl FnOnce(Ptr<'w>) -> &'w T) -> Ref<'w, T> {
        Ref {
            value: f(self.value),
            ticks: self.ticks,
        }
    }

    /// Specifies the reference type and converts self to a [`Ref`].
    ///
    /// Only works for `Sized` types. Use [`map_type`](Self::map_type) for
    /// `!Sized` types.
    ///
    /// # Safety
    /// `T` must be the erased pointee type for this [`UntypedRef`].
    #[inline(always)]
    pub unsafe fn with_type<T>(self) -> Ref<'w, T> {
        self.value.debug_assert_aligned::<T>();
        Ref {
            value: unsafe { self.value.as_ref() },
            ticks: self.ticks,
        }
    }

    /// # Safety
    /// `T` must be the erased pointee type for this [`UntypedRef`].
    #[inline(always)]
    pub unsafe fn into_res<T: Resource>(self) -> Res<'w, T> {
        self.value.debug_assert_aligned::<T>();
        Res {
            value: unsafe { self.value.as_ref() },
            ticks: self.ticks,
        }
    }

    /// # Safety
    /// `T` must be the erased pointee type for this [`UntypedRef`].
    #[inline(always)]
    pub unsafe fn into_non_send<T: NonSendResource>(self) -> NonSend<'w, T> {
        self.value.debug_assert_aligned::<T>();
        NonSend {
            value: unsafe { self.value.as_ref() },
            ticks: self.ticks,
        }
    }
}

impl core::fmt::Debug for UntypedRef<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("UntypedRef")
            .field(&self.value.as_ptr())
            .finish()
    }
}

// -----------------------------------------------------------------------------
// UntypedMut : Method Implementation

impl<'w> UntypedMut<'w> {
    /// Consumes self and returns the inner [`PtrMut`].
    ///
    /// This function does not set the change flag.
    #[inline(always)]
    pub fn into_inner(self) -> PtrMut<'w> {
        self.value
    }

    /// Returns a shorter-lived version of self.
    ///
    /// This function does not set the change flag.
    #[inline(always)]
    pub fn reborrow(&mut self) -> UntypedMut<'_> {
        UntypedMut {
            value: self.value.reborrow(),
            ticks: ComponentTicksMut {
                added: self.ticks.added,
                changed: self.ticks.changed,
                last_run: self.ticks.last_run,
                this_run: self.ticks.this_run,
            },
        }
    }

    /// Checks whether this value has changed since the given tick.
    #[inline]
    pub fn has_changed_since(&self, tick: Tick) -> bool {
        self.ticks.changed.is_newer_than(tick, self.ticks.this_run)
    }

    /// Converts self to a [`Mut`] by specifying the reference type via a function.
    ///
    /// This function is assumed to only change the type, not modify data.
    /// Modifying data through the mutable pointer in the closure is undefined behavior
    /// (data may be modified without triggering change events).
    ///
    /// Consider using [`with_type`](Self::with_type) instead for `Sized` types without
    /// complex operations.
    ///
    /// # Safety
    /// `T` must be the erased pointee type for this [`UntypedMut`].
    #[inline(always)]
    pub unsafe fn map_type<T: ?Sized>(self, f: impl FnOnce(PtrMut<'w>) -> &'w mut T) -> Mut<'w, T> {
        Mut {
            value: f(self.value),
            ticks: self.ticks,
        }
    }

    /// Specifies the reference type and converts self to a [`Mut`].
    ///
    /// This function does not set the change flag.
    ///
    /// Only works for `Sized` types. Use [`map_type`](Self::map_type) for
    /// `!Sized` types.
    ///
    /// # Safety
    /// `T` must be the erased pointee type for this [`UntypedMut`].
    #[inline(always)]
    pub unsafe fn with_type<T>(self) -> Mut<'w, T> {
        self.value.debug_assert_aligned::<T>();
        Mut {
            value: unsafe { self.value.consume() },
            ticks: self.ticks,
        }
    }

    /// # Safety
    ///
    /// `T` must be the erased pointee type for this [`UntypedMut`].
    #[inline(always)]
    pub unsafe fn into_res<T: Resource>(self) -> Mut<'w, T> {
        self.value.debug_assert_aligned::<T>();
        Mut {
            value: unsafe { self.value.consume() },
            ticks: self.ticks,
        }
    }

    /// # Safety
    ///
    /// `T` must be the erased pointee type for this [`UntypedMut`].
    #[inline(always)]
    pub unsafe fn into_non_send<T: NonSendResource>(self) -> NonSendMut<'w, T> {
        self.value.debug_assert_aligned::<T>();
        NonSendMut {
            value: unsafe { self.value.consume() },
            ticks: self.ticks,
        }
    }
}

impl core::fmt::Debug for UntypedMut<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("UntypedMut")
            .field(&self.value.as_ptr())
            .finish()
    }
}

// -----------------------------------------------------------------------------
// UntypedSliceRef : Method Implementation

impl<'w> UntypedSliceRef<'w> {
    /// Consumes self and returns the inner [`PtrMut`].
    #[inline]
    pub fn into_inner(self) -> Ptr<'w> {
        self.value
    }

    #[inline]
    pub fn len(&self) -> usize {
        debug_assert_eq!(self.ticks.added.len(), self.ticks.changed.len());
        self.ticks.added.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        debug_assert_eq!(self.ticks.added.len(), self.ticks.changed.len());
        self.ticks.added.len() == 0
    }

    /// # Safety
    /// `T` must be the erased pointee type for this [`UntypedSliceRef`].
    pub unsafe fn with_type<T>(self) -> SliceRef<'w, T> {
        self.value.debug_assert_aligned::<T>();
        debug_assert_eq!(self.ticks.added.len(), self.ticks.changed.len());
        SliceRef {
            value: unsafe {
                let data = self.value.as_ptr().cast::<T>();
                let len = self.ticks.added.len();
                core::slice::from_raw_parts(data, len)
            },
            ticks: self.ticks,
        }
    }
}

impl core::fmt::Debug for UntypedSliceRef<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("UntypedSliceRef")
            .field("ptr", &self.value.as_ptr())
            .field("len", &self.ticks.added.len())
            .finish()
    }
}

// -----------------------------------------------------------------------------
// UntypedSliceMut : Method Implementation

impl<'w> UntypedSliceMut<'w> {
    /// Consumes self and returns the inner [`PtrMut`].
    #[inline]
    pub fn into_inner(self) -> PtrMut<'w> {
        self.value
    }

    #[inline]
    pub fn len(&self) -> usize {
        debug_assert_eq!(self.ticks.added.len(), self.ticks.changed.len());
        self.ticks.added.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        debug_assert_eq!(self.ticks.added.len(), self.ticks.changed.len());
        self.ticks.added.len() == 0
    }

    /// # Safety
    /// `T` must be the erased pointee type for this [`UntypedSliceRef`].
    pub unsafe fn with_type<T>(self) -> SliceMut<'w, T> {
        self.value.debug_assert_aligned::<T>();
        debug_assert_eq!(self.ticks.added.len(), self.ticks.changed.len());
        SliceMut {
            value: unsafe {
                let data = self.value.as_ptr().cast::<T>();
                let len = self.ticks.added.len();
                core::slice::from_raw_parts_mut(data, len)
            },
            ticks: self.ticks,
        }
    }
}

impl core::fmt::Debug for UntypedSliceMut<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("UntypedSliceMut")
            .field("ptr", &self.value.as_ptr())
            .field("len", &self.ticks.added.len())
            .finish()
    }
}
