use core::any::Any;
use core::hash::{Hash, Hasher};

// Re-exported for use within `define_label!`
#[doc(hidden)]
pub use alloc::boxed::Box;

// -----------------------------------------------------------------------------
// DynEq

pub trait DynEq: Any {
    fn dyn_eq(&self, other: &dyn DynEq) -> bool;
}

impl<T: Any + Eq> DynEq for T {
    fn dyn_eq(&self, other: &dyn DynEq) -> bool {
        if let Some(other) = <dyn Any>::downcast_ref::<T>(other) {
            return self == other;
        }
        false
    }
}

// -----------------------------------------------------------------------------
// DynHash

pub trait DynHash {
    fn dyn_hash(&self, state: &mut dyn Hasher);
}

impl<T: Hash + Any> DynHash for T {
    fn dyn_hash(&self, mut state: &mut dyn Hasher) {
        T::hash(self, &mut state);
    }
}

// -----------------------------------------------------------------------------
// define_label

#[macro_export]
macro_rules! define_label {
    (
        $(#[$label_attr:meta])*
        $label_trait_name:ident,
        $interner_name:ident
    ) => {
        $crate::define_label!(
            $(#[$label_attr])*
            $label_trait_name,
            $interner_name,
            extra_methods: {},
            extra_methods_impl: {}
        );
    };
    (
        $(#[$label_attr:meta])*
        $label_trait_name:ident,
        $interner_name:ident,
        extra_methods: { $($trait_extra_methods:tt)* },
        extra_methods_impl: { $($interned_extra_methods_impl:tt)* }
    ) => {

        $(#[$label_attr])*
        pub trait $label_trait_name: ::core::marker::Send + ::core::marker::Sync +
            ::core::fmt::Debug + $crate::label::DynEq + $crate::label::DynHash
        {

            $($trait_extra_methods)*

            /// Clones this `
            #[doc = stringify!($label_trait_name)]
            ///`.
            fn dyn_clone(&self) -> $crate::label::Box<dyn $label_trait_name>;

            /// Returns an [`Interned`] value corresponding to `self`.
            #[inline]
            fn intern(&self) -> $crate::intern::Interned<dyn $label_trait_name>
            where
                Self: Sized
            {
                $interner_name.intern(self)
            }
        }

        #[diagnostic::do_not_recommend]
        impl $label_trait_name for $crate::intern::Interned<dyn $label_trait_name> {

            $($interned_extra_methods_impl)*

            #[inline(always)]
            fn dyn_clone(&self) -> $crate::label::Box<dyn $label_trait_name> {
                (**self).dyn_clone()
            }

            #[inline(always)]
            fn intern(&self) -> Self {
                *self
            }
        }

        impl ::core::cmp::PartialEq for dyn $label_trait_name {
            #[inline]
            fn eq(&self, other: &Self) -> bool {
                self.dyn_eq(other)
            }
        }

        impl ::core::cmp::Eq for dyn $label_trait_name {}

        impl ::core::hash::Hash for dyn $label_trait_name {
            #[inline]
            fn hash<H: ::core::hash::Hasher>(&self, state: &mut H) {
                self.dyn_hash(state);
            }
        }

        impl $crate::intern::Internable for dyn $label_trait_name {
            #[inline]
            fn leak(&self) -> &'static Self {
                $crate::label::Box::leak(self.dyn_clone())
            }

            #[inline]
            fn ref_eq(&self, other: &Self) -> bool {
                ::core::ptr::addr_eq(
                    ::core::ptr::from_ref::<Self>(self),
                    ::core::ptr::from_ref::<Self>(other)
                ) && self.type_id() == other.type_id()
            }

            #[inline]
            fn ref_hash<H: ::core::hash::Hasher>(&self, state: &mut H) {
                ::core::hash::Hash::hash(&self.type_id(), state);

                ::core::hash::Hash::hash(
                    &::core::ptr::from_ref::<Self>(self) as *const (),
                    state
                );
            }
        }

        static $interner_name: $crate::intern::Interner<dyn $label_trait_name> =
            $crate::intern::Interner::new();
    };
}
