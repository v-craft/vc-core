use crate::derive::{impl_reflect_opaque, impl_type_path};
use crate::impls::{impl_reflect_for_fixedhashmap, impl_reflect_for_fixedhashset};
use crate::impls::{impl_reflect_for_hashmap, impl_reflect_for_hashset};

// -----------------------------------------------------------------------------
// HashState Hasher

impl_type_path!(::vc_utils::hash::FixedHashState);
impl_type_path!(::vc_utils::hash::NoOpHashState);
impl_type_path!(::vc_utils::hash::SparseHashState);

impl_reflect_opaque!(
    ::vc_utils::hash::Hashed<V: Eq + PartialEq + Clone>
    (clone, hash, eq)
);

impl_type_path!(
    (in foldhash::fast as RandomState)
    ::vc_utils::hash::foldhash::fast::RandomState
);

impl_type_path!(
    (in foldhash::fast as FixedState)
    ::vc_utils::hash::foldhash::fast::FixedState
);

impl_type_path!(
    (in foldhash::quality as RandomState)
    ::vc_utils::hash::foldhash::quality::RandomState
);

impl_type_path!(
    (in foldhash::quality as FixedState)
    ::vc_utils::hash::foldhash::quality::FixedState
);

impl_type_path!(
    (in hashbrown as DefaultHashBuilder)
    ::vc_utils::hash::hashbrown::DefaultHashBuilder
);

// -----------------------------------------------------------------------------
// Fixed HashSet and HashMap

impl_type_path!(::vc_utils::hash::HashSet<T, S>);
impl_type_path!(::vc_utils::hash::HashMap<K, V, S>);

impl_reflect_for_hashset!(
    ::vc_utils::hash::HashSet<T, S>,
    ::vc_utils::hash::FixedHashState,
);

impl_reflect_for_hashmap!(
    ::vc_utils::hash::HashMap<K, V, S>,
    ::vc_utils::hash::FixedHashState,
);

// -----------------------------------------------------------------------------
// NoOp HashSet and HashMap

impl_type_path!(::vc_utils::hash::NoOpHashSet<T>);
impl_type_path!(::vc_utils::hash::NoOpHashMap<K, V>);

impl_reflect_for_fixedhashset!(::vc_utils::hash::NoOpHashSet<T>);

impl_reflect_for_fixedhashmap!(
    ::vc_utils::hash::NoOpHashMap<K, V>
);

// -----------------------------------------------------------------------------
// Sparse HashSet and HashMap

impl_type_path!(::vc_utils::hash::SparseHashSet<T>);
impl_type_path!(::vc_utils::hash::SparseHashMap<K, V>);

impl_reflect_for_fixedhashset!(::vc_utils::hash::SparseHashSet<T>);

impl_reflect_for_fixedhashmap!(
    ::vc_utils::hash::SparseHashMap<K, V>
);

// // -----------------------------------------------------------------------------
// // hashbrown HashSet and HashMap
//
// impl_type_path!(
//     (in hashbrown as HashSet)
//     ::vc_utils::hash::hashbrown::HashSet<T, S>
// );
//
// impl_type_path!(
//     (in hashbrown as HashMap)
//     ::vc_utils::hash::hashbrown::HashMap<K, V, S>
// );
//
// impl_reflect_for_hashset!(
//     ::vc_utils::hash::hashbrown::HashSet<T, S>,
//     ::vc_utils::hash::hashbrown::DefaultHashBuilder,
// );
//
// impl_reflect_for_hashmap!(
//     ::vc_utils::hash::hashbrown::HashMap<K, V, S>,
//     ::vc_utils::hash::hashbrown::DefaultHashBuilder,
// );
