use core::fmt::Debug;
use core::hash::{BuildHasher, Hash, Hasher};
use core::ops::Deref;

use hashbrown::hash_map::RawEntryMut;

use crate::hash::{FixedHashState, NoOpHashMap};

// -----------------------------------------------------------------------------
// Hashed

/// A pre-hashed value of a specific type.
///
/// Pre-hashing enables memoization of hashes that are expensive to compute.
///
/// It also enables faster [`PartialEq`] comparisons by short circuiting on hash
/// equality. See [`PreHashMap`] for a hashmap pre-configured to use `Hashed` keys.
pub struct Hashed<V> {
    hash: u64,
    value: V,
}

impl<V: Hash> Hashed<V> {
    /// Use the built-in fixed hash function to
    /// calculate the hash value.
    ///
    /// # Examples
    /// ```
    /// use vc_utils::hash::Hashed;
    ///
    /// let hashed = Hashed::with_hash(1, Hashed::hash_one(&1));
    /// // as same as
    /// let hashed = Hashed::new(1);
    /// ```
    #[inline]
    pub fn hash_one(value: &V) -> u64 {
        FixedHashState.hash_one(value)
    }
}

impl<V: Hash + Eq + Clone> Hashed<V> {
    /// Pre-hashes the given value using the [`FixedHashState`].
    ///
    /// # Examples
    ///
    /// ```
    /// use vc_utils::hash::Hashed;
    ///
    /// let hashed = Hashed::new(1);
    /// ```
    #[inline]
    pub fn new(value: V) -> Self {
        Self {
            hash: FixedHashState.hash_one(&value),
            value,
        }
    }
}

impl<V: Eq + Clone> Hashed<V> {
    /// Create a `Hashed` through given value and hash value.
    ///
    /// # Examples
    ///
    /// ```
    /// use vc_utils::hash::Hashed;
    ///
    /// let hashed = Hashed::with_hash(1, 61671341508);
    /// ```
    #[inline(always)]
    pub const fn with_hash(value: V, hash: u64) -> Self {
        Self { value, hash }
    }

    /// Return the pre-computed hash.
    ///
    /// # Examples
    ///
    /// ```
    /// use vc_utils::hash::Hashed;
    ///
    /// let hashed = Hashed::with_hash(1, 61671341508);
    /// assert_eq!(hashed.hash(), 61671341508);
    /// ```
    #[inline(always)]
    pub const fn hash(&self) -> u64 {
        self.hash
    }
}

impl<V> Hashed<V> {
    /// Extract internal value.
    ///
    /// # Examples
    ///
    /// ```
    /// use vc_utils::hash::Hashed;
    ///
    /// let hashed = Hashed::new(1);
    /// assert_eq!(hashed.into_inner(), 1);
    /// ```
    #[inline(always)]
    pub fn into_inner(self) -> V {
        self.value
    }
}

impl<V> Hash for Hashed<V> {
    #[inline]
    fn hash<R: Hasher>(&self, state: &mut R) {
        state.write_u64(self.hash);
    }
}

impl<V> Deref for Hashed<V> {
    type Target = V;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<V: PartialEq> PartialEq for Hashed<V> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash && self.value.eq(&other.value)
    }
}

impl<V: Eq> Eq for Hashed<V> {}

impl<V: Debug> Debug for Hashed<V> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Hashed")
            .field("hash", &self.hash)
            .field("value", &self.value)
            .finish()
    }
}

impl<V: Clone> Clone for Hashed<V> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            hash: self.hash,
            value: self.value.clone(),
        }
    }
}

impl<V: Copy> Copy for Hashed<V> {}

// -----------------------------------------------------------------------------
// PreHashMap

/// A [`NoOpHashMap`] pre-configured to use [`Hashed`] keys.
pub type PreHashMap<K, V> = NoOpHashMap<Hashed<K>, V>;

impl<K: Hash + Eq + Clone, V> PreHashMap<K, V> {
    /// Try to get or insert the value for the given hashed `key`.
    ///
    /// If the [`PreHashMap`] does not already contain the `key`,
    /// it will clone it and insert the value returned by `func`.
    #[inline]
    pub fn get_or_insert_with(&mut self, key: &Hashed<K>, func: impl FnOnce() -> V) -> &mut V {
        let entry = self
            .raw_entry_mut()
            .from_key_hashed_nocheck(key.hash(), key);

        match entry {
            RawEntryMut::Occupied(entry) => entry.into_mut(),
            RawEntryMut::Vacant(entry) => {
                let (_, value) = entry.insert_hashed_nocheck(key.hash(), key.clone(), func());
                value
            }
        }
    }
}
