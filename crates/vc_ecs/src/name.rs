use alloc::borrow::{Cow, ToOwned};
use alloc::string::String;
use core::fmt;
use core::hash::{BuildHasher, Hash, Hasher};
use core::ops::Deref;

use serde::de::{Error, Visitor};
use serde::{Deserialize, Deserializer};
use serde::{Serialize, Serializer};

use vc_reflect::derive::Reflect;
use vc_utils::hash::FixedHashState;

// -----------------------------------------------------------------------------
// Name

#[derive(Reflect, Clone)]
#[reflect(Opaque, full)]
pub struct Name {
    hash: u64, // Won't be serialized
    name: Cow<'static, str>,
}

impl Default for Name {
    #[inline(always)]
    fn default() -> Self {
        Name::new("")
    }
}

impl Name {
    #[inline(always)]
    fn update_hash(&mut self) {
        self.hash = FixedHashState.hash_one(&self.name);
    }

    #[inline]
    pub fn new(name: impl Into<Cow<'static, str>>) -> Self {
        let name = name.into();
        let mut name = Name { name, hash: 0 };
        name.update_hash();
        name
    }

    #[inline(always)]
    pub fn as_str(&self) -> &str {
        &self.name
    }

    #[inline(always)]
    pub fn mutate<F: FnOnce(&mut String)>(&mut self, f: F) {
        f(self.name.to_mut());
        self.update_hash();
    }

    #[inline]
    pub fn set(&mut self, name: impl Into<Cow<'static, str>>) {
        self.name = name.into();
        self.update_hash();
    }
}

impl Deref for Name {
    type Target = str;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        self.name.as_ref()
    }
}

impl Hash for Name {
    #[inline(always)]
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.hash);
    }
}

impl PartialEq for Name {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        if self.hash != other.hash {
            // Makes the common case of two strings not been equal very fast
            return false;
        }

        self.name.eq(&other.name)
    }
}

impl Eq for Name {}

impl PartialOrd for Name {
    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Name {
    #[inline(always)]
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.name.cmp(&other.name)
    }
}

impl fmt::Display for Name {
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.name, f)
    }
}

impl fmt::Debug for Name {
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self.name, f)
    }
}

impl Serialize for Name {
    #[inline]
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for Name {
    #[inline]
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct NameVisitor;

        impl<'de> Visitor<'de> for NameVisitor {
            type Value = Name;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str(core::any::type_name::<Name>())
            }

            #[inline]
            fn visit_str<E: Error>(self, v: &str) -> Result<Self::Value, E> {
                Ok(Name::new(v.to_owned()))
            }

            #[inline]
            fn visit_string<E: Error>(self, v: String) -> Result<Self::Value, E> {
                Ok(Name::new(v))
            }
        }

        deserializer.deserialize_str(NameVisitor)
    }
}

impl From<&str> for Name {
    #[inline(always)]
    fn from(name: &str) -> Self {
        Name::new(name.to_owned())
    }
}

impl From<String> for Name {
    #[inline(always)]
    fn from(name: String) -> Self {
        Name::new(name)
    }
}

impl AsRef<str> for Name {
    #[inline(always)]
    fn as_ref(&self) -> &str {
        &self.name
    }
}

impl From<&Name> for String {
    #[inline(always)]
    fn from(val: &Name) -> String {
        val.as_str().to_owned()
    }
}

impl From<Name> for String {
    #[inline(always)]
    fn from(val: Name) -> String {
        val.name.into_owned()
    }
}
