use core::any::Any;
use core::hash::Hash;
use core::ops::Deref;
use core::{fmt::Debug, hash::Hasher};
use vc_os::sync::{PoisonError, RwLock};

use vc_utils::hash::HashSet;

pub use alloc::boxed::Box;

// -----------------------------------------------------------------------------
// pool

#[expect(unsafe_code, reason = "sealed")]
mod pool {
    use vc_os::sync::{Mutex, PoisonError};
    use vc_utils::extra::PagePool;

    struct MemoryPool(PagePool<1024>);

    unsafe impl Sync for MemoryPool {}
    unsafe impl Send for MemoryPool {}

    static STR_POOL: Mutex<MemoryPool> = Mutex::new(MemoryPool(PagePool::new()));

    pub fn leak_str(value: &str) -> &'static str {
        let guard = STR_POOL.lock().unwrap_or_else(PoisonError::into_inner);
        unsafe {
            let ref_str = guard.0.alloc_str(value);
            core::mem::transmute::<&str, &'static str>(ref_str)
        }
    }
}

// -----------------------------------------------------------------------------
// Internable

pub trait Internable: Hash + Eq + 'static {
    /// Creates a static reference to `self`, possibly leaking memory.
    fn leak(&self) -> &'static Self;
    /// Returns `true` if the two references point to the same value.
    fn ref_eq(&self, other: &Self) -> bool;
    /// Feeds the reference to the hasher.
    fn ref_hash<H: Hasher>(&self, state: &mut H);
}

impl Internable for str {
    fn leak(&self) -> &'static Self {
        pool::leak_str(self)
    }

    fn ref_eq(&self, other: &Self) -> bool {
        self.as_ptr() == other.as_ptr() && self.len() == other.len()
    }

    fn ref_hash<H: Hasher>(&self, state: &mut H) {
        self.len().hash(state);
        self.as_ptr().hash(state);
    }
}

// -----------------------------------------------------------------------------
// Interned

pub struct Interned<T: ?Sized + Internable>(pub &'static T);

impl<T: ?Sized + Internable> Copy for Interned<T> {}
impl<T: ?Sized + Internable> Clone for Interned<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: ?Sized + Internable> Deref for Interned<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<T: ?Sized + Internable> PartialEq for Interned<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0.ref_eq(other.0)
    }
}

impl<T: ?Sized + Internable> Eq for Interned<T> {}

impl<T: ?Sized + Internable> Hash for Interned<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.ref_hash(state);
    }
}

impl<T: ?Sized + Internable + Debug> Debug for Interned<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.0.fmt(f)
    }
}

impl<T: ?Sized + Internable> From<&Interned<T>> for Interned<T> {
    fn from(value: &Interned<T>) -> Self {
        *value
    }
}

// -----------------------------------------------------------------------------
// Interned

pub struct Interner<T: ?Sized + 'static>(RwLock<HashSet<&'static T>>);

impl<T: ?Sized> Interner<T> {
    /// Creates a new empty interner
    pub const fn new() -> Self {
        Self(RwLock::new(HashSet::new()))
    }
}

impl<T: ?Sized> Default for Interner<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: ?Sized + Internable> Interner<T> {
    /// Return the [`Interned<T>`] corresponding to `value`.
    ///
    /// If it is called the first time for `value`, it will possibly leak the value and return an
    /// [`Interned<T>`] using the obtained static reference. Subsequent calls for the same `value`
    /// will return [`Interned<T>`] using the same static reference.
    pub fn intern(&self, value: &T) -> Interned<T> {
        {
            let set = self.0.read().unwrap_or_else(PoisonError::into_inner);

            if let Some(val) = set.get(value) {
                return Interned(*val);
            }
        }

        {
            let mut set = self.0.write().unwrap_or_else(PoisonError::into_inner);

            let val = set.get_or_insert_with(value, |_| value.leak());
            Interned(*val)
        }
    }
}

// -----------------------------------------------------------------------------
// Dyn Hash/Eq

pub trait DynEq: Any {
    fn dyn_eq(&self, other: &dyn DynEq) -> bool;
}

pub trait DynHash: Any {
    fn dyn_hash(&self, state: &mut dyn Hasher);
}

impl<T: Any + Eq> DynEq for T {
    fn dyn_eq(&self, other: &dyn DynEq) -> bool {
        if let Some(other) = <dyn Any>::downcast_ref::<T>(other) {
            self == other
        } else {
            false
        }
    }
}

impl<T: Any + Hash> DynHash for T {
    fn dyn_hash(&self, mut state: &mut dyn Hasher) {
        T::hash(self, &mut state);
        self.type_id().hash(&mut state);
    }
}

// -----------------------------------------------------------------------------
// Label

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
        pub trait $label_trait_name: Send + Sync + ::core::fmt::Debug + $crate::label::DynEq + $crate::label::DynHash {

            $($trait_extra_methods)*

            /// Clones this `
            #[doc = stringify!($label_trait_name)]
            ///`.
            fn dyn_clone(&self) -> $crate::label::Box<dyn $label_trait_name>;

            /// Returns an [`Interned`] value corresponding to `self`.
            fn intern(&self) -> $crate::label::Interned<dyn $label_trait_name>
            where
                Self: Sized
            {
                $interner_name.intern(self)
            }
        }

        #[diagnostic::do_not_recommend]
        impl $label_trait_name for $crate::label::Interned<dyn $label_trait_name> {

            $($interned_extra_methods_impl)*

            fn dyn_clone(&self) -> $crate::label::Box<dyn $label_trait_name> {
                (**self).dyn_clone()
            }

            fn intern(&self) -> Self {
                *self
            }
        }

        impl PartialEq for dyn $label_trait_name {
            fn eq(&self, other: &Self) -> bool {
                self.dyn_eq(other)
            }
        }

        impl Eq for dyn $label_trait_name {}

        impl ::core::hash::Hash for dyn $label_trait_name {
            fn hash<H: ::core::hash::Hasher>(&self, state: &mut H) {
                self.dyn_hash(state);
            }
        }

        impl $crate::label::Internable for dyn $label_trait_name {
            fn leak(&self) -> &'static Self {
                $crate::label::Box::leak(self.dyn_clone())
            }

            fn ref_eq(&self, other: &Self) -> bool {
                use ::core::ptr;

                // Test that both the type id and pointer address are equivalent.
                self.type_id() == other.type_id()
                    && ptr::addr_eq(ptr::from_ref::<Self>(self), ptr::from_ref::<Self>(other))
            }

            fn ref_hash<H: ::core::hash::Hasher>(&self, state: &mut H) {
                use ::core::hash::Hash;

                // Hash the type id...
                self.type_id().hash(state);

                // ...and the pointer address.
                // Cast to a unit `()` first to discard any pointer metadata.
                ::core::ptr::from_ref::<Self>(self).cast::<()>().hash(state);
            }
        }

        static $interner_name: $crate::label::Interner<dyn $label_trait_name> =
            $crate::label::Interner::new();
    };
}

// -----------------------------------------------------------------------------
// Tests

#[cfg(test)]
mod tests {
    use core::hash::{BuildHasher, Hash, Hasher};
    use vc_utils::hash::FixedHashState;

    use super::{Internable, Interned, Interner};

    #[test]
    fn zero_sized_type() {
        #[derive(PartialEq, Eq, Hash, Debug)]
        pub struct A;

        impl Internable for A {
            fn leak(&self) -> &'static Self {
                &A
            }

            fn ref_eq(&self, other: &Self) -> bool {
                core::ptr::eq(self, other)
            }

            fn ref_hash<H: Hasher>(&self, state: &mut H) {
                core::ptr::hash(self, state);
            }
        }

        let interner = Interner::default();
        let x = interner.intern(&A);
        let y = interner.intern(&A);
        assert_eq!(x, y);
    }

    #[test]
    fn fieldless_enum() {
        #[derive(PartialEq, Eq, Hash, Debug, Clone)]
        pub enum A {
            X,
            Y,
        }

        impl Internable for A {
            fn leak(&self) -> &'static Self {
                match self {
                    A::X => &A::X,
                    A::Y => &A::Y,
                }
            }

            fn ref_eq(&self, other: &Self) -> bool {
                core::ptr::eq(self, other)
            }

            fn ref_hash<H: Hasher>(&self, state: &mut H) {
                core::ptr::hash(self, state);
            }
        }

        let interner = Interner::default();
        let x = interner.intern(&A::X);
        let y = interner.intern(&A::Y);
        assert_ne!(x, y);
    }

    #[test]
    fn static_sub_strings() {
        let str = "ABC ABC";
        let a = &str[0..3];
        let b = &str[4..7];
        // Same contents
        assert_eq!(a, b);
        let x = Interned(a);
        let y = Interned(b);
        // Different pointers
        assert_ne!(x, y);
        let interner = Interner::default();
        let x = interner.intern(a);
        let y = interner.intern(b);
        // Same pointers returned by interner
        assert_eq!(x, y);
    }

    #[test]
    fn same_interned_instance() {
        let a = Interned("A");
        let b = a;

        assert_eq!(a, b);

        let hash_a = FixedHashState.hash_one(a);
        let hash_b = FixedHashState.hash_one(b);

        assert_eq!(hash_a, hash_b);
    }

    #[test]
    fn same_interned_content() {
        let a = Interned::<str>(Internable::leak("A"));
        let b = Interned::<str>(Internable::leak("A"));

        assert_ne!(a, b);
    }

    #[test]
    fn different_interned_content() {
        let a = Interned::<str>("A");
        let b = Interned::<str>("B");

        assert_ne!(a, b);
    }
}
