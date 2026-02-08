#![expect(unsafe_code, reason = "`core::mem::transmute` for better performance.")]

use core::fmt;
use core::hash::Hash;
use core::mem;
use core::num::NonZeroU32;

use serde::{Deserialize, Serialize};
use vc_reflect::derive::Reflect;

use super::{EntityGeneration, EntityId};

// -----------------------------------------------------------------------------
// Entity

#[derive(Reflect, Clone, Copy)]
#[reflect(Opaque, serde, clone, hash, eq, debug)]
#[repr(C, align(8))]
pub struct Entity {
    #[cfg(target_endian = "little")]
    id: EntityId,
    generation: EntityGeneration,
    #[cfg(target_endian = "big")]
    id: EntityId,
}

impl Entity {
    const _STATIC_ASSERT_: () = const {
        const ENTITY: Entity = Entity::from_bits(11);
        assert!(ENTITY.index_u32() == 11);
    };

    pub const PLACEHOLDER: Self = Self::from_id(EntityId::PLACEHOLDER);

    #[inline(always)]
    pub const fn new(id: EntityId, generation: EntityGeneration) -> Entity {
        Self { id, generation }
    }

    /// Equivalent to `self.id().index()`. See [`Self::index`] for details.
    #[inline(always)]
    pub const fn index(self) -> usize {
        self.id.index()
    }

    #[inline(always)]
    pub const fn index_u32(self) -> u32 {
        self.id.index_u32()
    }

    #[inline(always)]
    pub const fn id(self) -> EntityId {
        self.id
    }

    #[inline(always)]
    pub const fn generation(self) -> EntityGeneration {
        self.generation
    }

    #[inline(always)]
    pub const fn from_id(id: EntityId) -> Entity {
        Self {
            id,
            generation: EntityGeneration::FIRST,
        }
    }

    #[inline(always)]
    pub const fn from_u32(index: u32) -> Option<Entity> {
        match NonZeroU32::new(index) {
            Some(inner) => Some(Self::from_id(EntityId::new(inner))),
            None => None,
        }
    }

    #[inline(always)]
    pub const fn to_bits(self) -> u64 {
        unsafe { mem::transmute::<Entity, u64>(self) }
    }

    #[inline(always)]
    pub const fn from_bits(bits: u64) -> Self {
        unsafe {
            let entity = mem::transmute::<u64, Entity>(bits);
            assert!(mem::transmute::<EntityId, u32>(entity.id) != 0);
            entity
        }
    }

    #[inline(always)]
    pub const unsafe fn from_bits_unchecked(bits: u64) -> Self {
        unsafe { mem::transmute::<u64, Entity>(bits) }
    }
}

impl PartialEq for Entity {
    #[inline(always)]
    fn eq(&self, other: &Entity) -> bool {
        self.to_bits() == other.to_bits()
    }
}

impl Eq for Entity {}

impl fmt::Display for Entity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self == &Self::PLACEHOLDER {
            f.pad("PLACEHOLDER")
        } else {
            f.pad(&alloc::format!("{}v{}", self.index(), self.generation()))
        }
    }
}

impl fmt::Debug for Entity {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl PartialOrd for Entity {
    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Entity {
    #[inline(always)]
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.to_bits().cmp(&other.to_bits())
    }
}

impl Hash for Entity {
    #[inline(always)]
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        state.write_u64(self.to_bits());
    }
}

impl Serialize for Entity {
    #[inline(always)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u64(self.to_bits())
    }
}

impl<'de> Deserialize<'de> for Entity {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;

        let bits: u64 = Deserialize::deserialize(deserializer)?;

        unsafe {
            let entity = mem::transmute::<u64, Entity>(bits);
            if mem::transmute::<EntityId, u32>(entity.id) != 0 {
                return Ok(entity);
            }
            Err(Error::custom(
                "Attempting to deserialize an invalid entity.",
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Entity;

    #[test]
    fn entity_is_u64() {
        assert_eq!(
            ::core::mem::size_of::<Entity>(),
            ::core::mem::size_of::<u64>()
        );
        assert_eq!(
            Entity::from_bits(123456789012_u64).to_bits(),
            123456789012_u64
        );
    }
}
