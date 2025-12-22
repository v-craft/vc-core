//! Provide serialization and deserialization support for the reflection system.
//!
//! This module primarily serves two purposes:
//!
//! 1. Enables serialization and deserialization under type-erased conditions
//! 2. Allows composite types to be serialized and deserialized based on the reflection system's
//!    type information, even without implementing `serde` traits
//!
//! # Overview
//!
//! ## Serialization
//!
//! - [`TypeTraitSerialize`]: Stores function pointers that enable dynamic types to invoke
//!   `serde`'s serialization implementations.
//! - [`SerializeProcessor`]: Serialization processor that allows users to customize
//!   serialization behavior.
//! - [`SerializeDriver`]: Standard serializer that follows a priority-based dispatch strategy.
//!     - First attempts to use [`SerializeProcessor`]; if supported, returns its result immediately.
//!     - Then queries and invokes [`TypeTraitSerialize`] if available.
//!     - Finally falls back to reflection-based serialization (unavailable for Opaque types).
//! - [`ReflectSerializeDriver`]: Wraps serialized data with type path mapping to preserve type information.
//!     - Only the outermost layer includes type paths; inner data types are inferred from field names,
//!       using [`SerializeDriver`] internally.
//!
//! See code examples in [`SerializeDriver`] and [`ReflectSerializeDriver`].
//!
//! ## Deserialization
//!
//! - [`TypeTraitDeserialize`]: Stores function pointers that enable dynamic types to invoke
//!   `serde`'s deserialization implementations.
//! - [`DeserializeProcessor`]: Deserialization processor that allows users to customize
//!   deserialization behavior.
//! - [`DeserializeDriver`]: Standard deserializer that follows a priority-based dispatch strategy.
//!     - First attempts to use [`DeserializeProcessor`]; if supported, returns its result immediately.
//!     - Then queries and invokes [`TypeTraitDeserialize`] if available.
//!     - Finally falls back to reflection-based deserialization (unavailable for Opaque types).
//!     - Corresponds to [`SerializeDriver`]; requires explicit [`TypeMeta`] since data lacks type information.
//! - [`ReflectDeserializeDriver`]: Parses data with type path mapping (corresponds to [`ReflectSerializeDriver`]).
//!     - Type paths embedded in the data allow lookup of [`TypeMeta`] from the registry,
//!       eliminating the need for manual specification.
//!     - Only the outermost layer requires type paths; inner data types are inferred from field names,
//!       using [`DeserializeDriver`] internally.
//!
//! See code examples in [`DeserializeDriver`] and [`ReflectDeserializeDriver`].
//!
//! ## Field Skipping
//!
//! A special attribute [`SkipSerde`] enables skipping fields during both serialization and deserialization.
//!
//! ### Scope of Effectiveness
//!
//! This attribute **only** works with the default implementation (reflection-based serialization/deserialization).
//! It has **no effect** when custom processors are provided or when types have native `serde` implementations.
//!
//! ### Safety Requirements
//!
//! The attribute can be applied to:
//! - Fields within structs, tuple structs, and enum variants
//!
//! **Cannot** be applied to:
//! - Fields of newtype structs/enums (single-field tuple structs or enum tuple variants)
//!
//! See code examples in [`SkipSerde`].
//!
//! [`TypeMeta`]: crate::registry::TypeMeta
//! [`TypeTraitDeserialize`]: crate::registry::TypeTraitDeserialize
//! [`TypeTraitSerialize`]: crate::registry::TypeTraitSerialize

// -----------------------------------------------------------------------------
// Debug utils

crate::cfg::debug! {
    mod info_stack;
    use info_stack::TypeInfoStack;
}

// -----------------------------------------------------------------------------
// Modules

mod de;
mod ser;
mod skip_field;

// -----------------------------------------------------------------------------
// Exports

pub use de::{DeserializeDriver, DeserializeProcessor, ReflectDeserializeDriver};
pub use ser::{ReflectSerializeDriver, SerializeDriver, SerializeProcessor};
pub use skip_field::SkipSerde;
