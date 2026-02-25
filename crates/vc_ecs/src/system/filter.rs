//! 基于实体原型的过滤器以及访问标记器
//!
//! 仅基于实体原型，因此只能通过是否存在指定的组件进行过滤。

use alloc::boxed::Box;
use alloc::collections::BTreeSet;
use alloc::vec::Vec;
use core::hash::{Hash, Hasher};
use core::{fmt::Debug, hash::BuildHasher};

use vc_utils::hash::{FixedHashState, SparseHashSet};

use crate::component::ComponentId;

// -----------------------------------------------------------------------------
// FilterParam

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

    pub fn is_disjoint(&self, other: &Self) -> bool {
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

impl Hash for FilterParam {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.hash);
    }
}

#[derive(Default, Clone)]
pub struct FilterData {
    nothing: bool,
    entity_mut: bool, // holding `EntityMut`
    entity_ref: bool, // holding `EntityRef`
    reading: SparseHashSet<ComponentId>,
    writing: SparseHashSet<ComponentId>,
}

impl FilterData {
    pub const fn new() -> Self {
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
