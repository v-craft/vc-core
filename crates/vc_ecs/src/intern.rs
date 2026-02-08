use alloc::borrow::ToOwned;
use alloc::boxed::Box;
use core::fmt::Debug;
use core::hash::Hash;
use core::ops::Deref;
use vc_reflect::derive::Reflect;

use vc_os::sync::{PoisonError, RwLock};
use vc_utils::hash::{FixedHashState, HashSet};

// -----------------------------------------------------------------------------
// Internable

pub trait Internable: Hash + Eq {
    fn leak(&self) -> &'static Self;
    fn ref_eq(&self, other: &Self) -> bool;
    fn ref_hash<H: core::hash::Hasher>(&self, state: &mut H);
}

impl Internable for str {
    fn leak(&self) -> &'static Self {
        let str = self.to_owned().into_boxed_str();
        Box::leak(str)
    }

    #[inline]
    fn ref_eq(&self, other: &Self) -> bool {
        self.as_ptr() == other.as_ptr() && self.len() == other.len()
    }

    #[inline]
    fn ref_hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.len().hash(state);
        self.as_ptr().hash(state);
    }
}

// -----------------------------------------------------------------------------
// Interned

/// An interned value. Will stay valid until the end of the program and will not drop.
#[derive(Reflect)]
#[reflect(clone, hash, eq)]
pub struct Interned<T: ?Sized + Internable + 'static>(pub &'static T);

impl<T: ?Sized + Internable> Deref for Interned<T> {
    type Target = T;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<T: ?Sized + Internable> Clone for Interned<T> {
    #[inline(always)]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: ?Sized + Internable> Copy for Interned<T> {}

impl<T: ?Sized + Internable> PartialEq for Interned<T> {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.0.ref_eq(other.0)
    }
}

impl<T: ?Sized + Internable> Eq for Interned<T> {}

impl<T: ?Sized + Internable> Hash for Interned<T> {
    #[inline(always)]
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.0.ref_hash(state);
    }
}

impl<T: ?Sized + Internable + Debug> Debug for Interned<T> {
    #[inline(always)]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.0.fmt(f)
    }
}

impl<T: ?Sized + Internable> From<&Interned<T>> for Interned<T> {
    #[inline(always)]
    fn from(value: &Interned<T>) -> Self {
        *value
    }
}

// -----------------------------------------------------------------------------
// Interner

pub struct Interner<T: ?Sized + 'static>(RwLock<HashSet<&'static T>>);

impl<T: ?Sized> Default for Interner<T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<T: ?Sized> Interner<T> {
    /// Creates a new empty interner
    pub const fn new() -> Self {
        Self(RwLock::new(HashSet::with_hasher(FixedHashState)))
    }
}

impl<T: Internable + ?Sized> Interner<T> {
    /// Return the [`Interned<T>`] corresponding to `value`.
    pub fn intern(&self, value: &T) -> Interned<T> {
        {
            let set = self.0.read().unwrap_or_else(PoisonError::into_inner);

            if let Some(value) = set.get(value) {
                return Interned(*value);
            }
        }

        {
            let mut set = self.0.write().unwrap_or_else(PoisonError::into_inner);

            if let Some(value) = set.get(value) {
                Interned(*value)
            } else {
                let leaked = value.leak();
                set.insert(leaked);
                Interned(leaked)
            }
        }
    }
}
