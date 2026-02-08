use alloc::boxed::Box;
use alloc::collections::BTreeSet;
use alloc::vec::Vec;
use core::hash::{Hash, Hasher};
use core::{fmt::Debug, hash::BuildHasher};

use fixedbitset::FixedBitSet;
use vc_utils::hash::{FixedHashState, NoOpHashMap, SparseHashSet};

use crate::component::ComponentId;
use crate::resource::ResourceId;

#[derive(Debug, Default, Clone)]
pub struct FilterParamBuilder {
    // We use BTreeSet to ensure it's ordering.
    with: BTreeSet<ComponentId>,
    without: BTreeSet<ComponentId>,
}

impl FilterParamBuilder {
    pub const fn new() -> Self {
        Self {
            with: BTreeSet::new(),
            without: BTreeSet::new(),
        }
    }

    pub fn with(&mut self, id: ComponentId) {
        self.with.insert(id);
    }

    pub fn without(&mut self, id: ComponentId) {
        self.without.insert(id);
    }

    pub fn merge(&self, other: &Self) -> Option<FilterParamBuilder> {
        if self.with.is_disjoint(&other.without) && other.with.is_disjoint(&self.without) {
            let mut with = self.with.clone();
            with.extend(&other.with);
            let mut without = self.without.clone();
            without.extend(&other.without);
            Some(FilterParamBuilder { with, without })
        } else {
            None
        }
    }

    pub fn build(self) -> Option<FilterParam> {
        if self.with.is_disjoint(&self.without) {
            let with_len = self.with.len();
            let without_len = self.without.len();
            // ComponentId <= u32::MAX, ↓ length overflow is impossible
            let mut vec = Vec::with_capacity(with_len + without_len);
            vec.extend(self.with);
            vec.extend(self.without);
            let params = vec.into_boxed_slice();

            let mut hasher = FixedHashState.build_hasher();
            with_len.hash(&mut hasher);
            params.hash(&mut hasher);
            let hash = hasher.finish();

            Some(FilterParam {
                hash,
                with_len,
                params,
            })
        } else {
            None
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct FilterParam {
    hash: u64,
    with_len: usize,
    params: Box<[ComponentId]>,
}

impl FilterParam {
    pub fn with(&self) -> &[ComponentId] {
        &self.params[0..self.with_len]
    }

    pub fn without(&self) -> &[ComponentId] {
        &self.params[self.with_len..]
    }
}

impl Hash for FilterParam {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.hash);
    }
}

impl FilterParam {
    fn is_disjoint(&self, other: &Self) -> bool {
        use core::mem::transmute;
        // `SliceContains` has SIMD optimization for u32
        let x_without = unsafe { transmute::<&[ComponentId], &[u32]>(self.without()) };
        let y_without = unsafe { transmute::<&[ComponentId], &[u32]>(other.without()) };
        let x_with = unsafe { transmute::<&[ComponentId], &[u32]>(self.with()) };
        let y_with = unsafe { transmute::<&[ComponentId], &[u32]>(other.with()) };

        // Although the slice is sorted, we assume the params
        // is usually small, so the `contains` is faster.
        x_without.iter().any(|id| y_with.contains(id))
            || y_without.iter().any(|id| x_with.contains(id))
    }
}

#[derive(Default, Clone)]
pub struct FilterData {
    pub(crate) nothing: bool,
    pub(crate) entity_mut: bool, // holding `EntityMut`
    pub(crate) entity_ref: bool, // holding `EntityRef`
    pub(crate) reading: SparseHashSet<ComponentId>,
    pub(crate) writing: SparseHashSet<ComponentId>,
}

impl FilterData {
    const fn new() -> Self {
        Self {
            nothing: true,
            entity_mut: false,
            entity_ref: false,
            reading: SparseHashSet::new(),
            writing: SparseHashSet::new(),
        }
    }

    pub fn can_entity_ref(&self) -> bool {
        !self.entity_mut && self.writing.is_empty()
    }

    pub fn can_entity_mut(&self) -> bool {
        !self.entity_mut && !self.entity_ref && self.reading.is_empty() && self.writing.is_empty()
    }

    pub fn can_reading(&self, id: ComponentId) -> bool {
        self.entity_ref || (!self.entity_mut && !self.writing.contains(&id))
    }

    pub fn can_writing(&self, id: ComponentId) -> bool {
        !self.entity_mut && !self.entity_ref && !self.reading.contains(&id)
    }

    pub fn set_entity_ref(&mut self) {
        self.nothing = false;
        self.entity_ref = true;
        self.reading = SparseHashSet::new();
    }

    pub fn set_entity_mut(&mut self) {
        self.nothing = false;
        self.entity_mut = true;
        // ↓ useless, see `can_entity_mut` .
        // self.reading = SparseHashSet::new();
        // self.writing = SparseHashSet::new();
    }

    pub fn set_reading(&mut self, id: ComponentId) {
        self.nothing = false;
        if !self.entity_ref {
            self.reading.insert(id);
        }
    }

    pub fn set_writing(&mut self, id: ComponentId) {
        self.nothing = false;
        self.reading.insert(id);
        self.writing.insert(id);
    }

    pub fn parallelizable(&self, other: &Self) -> bool {
        if self.entity_mut {
            return other.nothing;
        }
        if other.entity_mut {
            return self.nothing;
        }
        if self.entity_ref {
            return other.writing.is_empty();
        }
        if other.entity_ref {
            return self.writing.is_empty();
        }
        self.writing.is_disjoint(&other.reading) && other.writing.is_disjoint(&self.reading)
    }

    pub fn merge(&mut self, other: &Self) {
        self.nothing &= other.nothing;
        self.entity_mut |= other.entity_mut;
        self.entity_ref |= other.entity_ref;
        if self.entity_mut || self.entity_ref {
            self.reading = SparseHashSet::new();
            // self.writing = BTreeSet::new();
        } else {
            self.reading.extend(&other.reading);
            self.writing.extend(&other.writing);
        }
    }
}

#[derive(Default)]
pub struct AccessTable {
    pub(crate) world_mut: bool,          // holding `&mut world`
    pub(crate) world_ref: bool,          // holding `&world`
    pub(crate) may_reading: FixedBitSet, // combined components reading
    pub(crate) may_writing: FixedBitSet, // combined components writing
    pub(crate) res_reading: FixedBitSet, // resource reading
    pub(crate) res_writing: FixedBitSet, // resource writing
    pub(crate) filter: NoOpHashMap<FilterParam, FilterData>,
}

// `#[derive(Clone)]` does not generate optimized `clone_from`.
impl Clone for AccessTable {
    fn clone(&self) -> Self {
        Self {
            world_mut: self.world_mut,
            world_ref: self.world_ref,
            may_reading: self.may_reading.clone(),
            may_writing: self.may_writing.clone(),
            res_reading: self.res_reading.clone(),
            res_writing: self.res_writing.clone(),
            filter: self.filter.clone(),
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.world_mut = source.world_mut;
        self.world_ref = source.world_ref;
        self.may_reading.clone_from(&source.may_reading);
        self.may_writing.clone_from(&source.may_writing);
        self.res_reading.clone_from(&source.res_reading);
        self.res_writing.clone_from(&source.res_writing);
        self.filter.clone_from(&source.filter);
    }
}

impl Debug for AccessTable {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        struct FormattedBitSet<'a>(&'a FixedBitSet);
        impl Debug for FormattedBitSet<'_> {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.debug_list().entries(self.0.ones()).finish()
            }
        }

        f.debug_struct("AccessTable")
            .field("world_mut", &self.world_mut)
            .field("world_ref", &self.world_ref)
            .field("may_reading", &FormattedBitSet(&self.may_reading))
            .field("may_writing", &FormattedBitSet(&self.may_writing))
            .field("res_reading", &FormattedBitSet(&self.res_reading))
            .field("res_writing", &FormattedBitSet(&self.res_writing))
            .finish()
    }
}

impl AccessTable {
    /// Creates an empty [`Access`] collection.
    pub const fn new() -> Self {
        Self {
            world_mut: false,
            world_ref: false,
            may_reading: FixedBitSet::new(),
            may_writing: FixedBitSet::new(),
            res_reading: FixedBitSet::new(),
            res_writing: FixedBitSet::new(),
            filter: NoOpHashMap::new(),
        }
    }

    pub fn can_world_mut(&self) -> bool {
        !self.world_mut
            && !self.world_ref
            && self.may_reading.is_clear()
            && self.may_writing.is_clear()
            && self.res_reading.is_clear()
            && self.res_writing.is_clear()
    }

    pub fn can_world_ref(&self) -> bool {
        self.world_ref
            || (!self.world_mut && self.may_writing.is_clear() && self.res_writing.is_clear())
    }

    pub fn set_world_mut(&mut self) {
        *self = const { Self::new() };
        self.world_mut = true;
    }

    pub fn set_world_ref(&mut self) {
        *self = const { Self::new() };
        self.world_ref = true;
    }

    pub fn can_reading_res(&self, id: ResourceId) -> bool {
        self.world_ref || (!self.world_mut && !self.res_writing.contains(id.index()))
    }

    pub fn can_writing_res(&self, id: ResourceId) -> bool {
        !self.world_ref && !self.world_mut && !self.res_reading.contains(id.index())
    }

    pub fn set_reading_res(&mut self, id: ResourceId) {
        if !self.world_ref {
            self.res_reading.grow_and_insert(id.index());
        }
    }

    pub fn set_writing_res(&mut self, id: ResourceId) {
        let index = id.index();
        self.res_reading.grow_and_insert(index);
        self.res_writing.grow_and_insert(index);
    }

    pub fn can_reading_in(&self, id: ComponentId, filter: &FilterParam) -> bool {
        if self.world_mut || self.world_ref {
            return self.world_ref;
        }
        if !self.may_writing.contains(id.index()) {
            return true;
        }
        self.filter.iter().all(|(k, v)| {
            if k.is_disjoint(filter) {
                true
            } else {
                v.can_reading(id)
            }
        })
    }

    pub fn can_writing_in(&self, id: ComponentId, filter: &FilterParam) -> bool {
        if self.world_mut || self.world_ref {
            return false;
        }
        if !self.may_reading.contains(id.index()) {
            return true;
        }
        self.filter.iter().any(|(k, v)| {
            if k.is_disjoint(filter) {
                true
            } else {
                v.can_writing(id)
            }
        })
    }

    pub fn set_reading_in(&mut self, id: ComponentId, filter: &FilterParam) {
        if self.world_ref {
            return;
        }
        self.may_reading.grow_and_insert(id.index());
        if let Some(v) = self.filter.get_mut(filter) {
            v.set_reading(id);
        } else {
            let mut data = FilterData::new();
            data.set_reading(id);
            self.filter.insert(filter.clone(), data);
        }
    }

    pub fn set_writing_in(&mut self, id: ComponentId, filter: &FilterParam) {
        self.may_reading.grow_and_insert(id.index());
        self.may_writing.grow_and_insert(id.index());
        if let Some(v) = self.filter.get_mut(filter) {
            v.set_writing(id);
        } else {
            let mut data = FilterData::new();
            data.set_writing(id);
            self.filter.insert(filter.clone(), data);
        }
    }

    pub fn parallelizable(&self, other: &Self) -> bool {
        if self.world_mut || other.world_mut {
            return false;
        }
        if self.world_ref && other.world_ref {
            return true;
        }
        if self.world_ref {
            return other.res_writing.is_empty() && other.may_writing.is_clear();
        }
        if other.world_ref {
            return self.res_writing.is_empty() && self.may_writing.is_clear();
        }
        if !self.res_writing.is_disjoint(&other.res_reading)
            || !other.res_writing.is_disjoint(&self.res_reading)
        {
            return false;
        }
        if self.may_writing.is_disjoint(&other.may_reading)
            && other.may_writing.is_disjoint(&self.may_reading)
        {
            return true;
        }
        self.filter.iter().all(|(k, v)| {
            other.filter.iter().all(|(x, y)| {
                if k.is_disjoint(x) {
                    true
                } else {
                    v.parallelizable(y)
                }
            })
        })
    }

    pub fn merge(&mut self, other: &Self) {
        if self.world_mut || self.world_ref {
            return;
        }
        if other.world_mut || other.world_ref {
            self.clone_from(other);
            return;
        }

        self.may_reading.union_with(&other.may_reading);
        self.may_writing.union_with(&other.may_writing);
        self.res_reading.union_with(&other.res_reading);
        self.res_writing.union_with(&other.res_writing);

        other.filter.iter().for_each(|(k, v)| {
            if let Some(data) = self.filter.get_mut(k) {
                data.merge(v);
            } else {
                self.filter.insert(k.clone(), v.clone());
            }
        });
    }
}
