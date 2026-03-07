# Runtime reflection system for Rust.

This library implements a dynamic reflection system in Rust, designed to provide
comprehensive runtime type information and data manipulation capabilities.

While it's a general-purpose reflection system suitable for various scenarios,
it's specifically designed for the VoidCraft Engine and may include
platform-specific dependencies from VoidCraft that could be redundant in
non-game-engine contexts.

## Goals

As a dynamic reflection system, this library aims to support:

- **Runtime Type Information**:
    - Basic information: type names, TypeId, field lists, generic parameters
    - Custom attributes: similar to C# attributes, allowing user-defined metadata on types
    - Type documentation (optional): useful for game engine editors and tools
    - See more information in [`vc_reflect::info`].

- **Data Manipulation**:
    - Type erasure: achieve effects similar to `Object` in other languages through trait objects
    - Specialized interfaces through reflection subtraits: `Struct`, `Enum`, etc.
    - Dynamic object composition with ability to apply to concrete types when needed
    - See more information in [`vc_reflect::ops`] and [`vc_reflect::Reflect`].

- **Type Registration**:
    - Metadata: type metadata containing both type information and available function pointers
    - Registry: storage system for metadata enabling type information retrieval without instances
    - Auto-registration (optional): type registration through static initialization
    - See more information in [`vc_reflect::registry`].

- **Trait Reflection**:
    - Trait reflection based on registration system, enabling dynamic trait object retrieval
    - See more information in [`registry::TypeTrait`] and [`derive::reflect_cast`]

- **Reflection Macros**:
    - Automatic generation of reflection implementations for types
    - See more information in [`vc_reflect::derive`].

- **(De)Serialization**:
    - (De)Serialization system based on registry, allowing types without explicit `Serialize`/`Deserialize` implementations
    - See more information in [`vc_reflect::serde`].

- **Path-Based Access**:
    - Multi-level data access via string paths (struct fields, array elements, etc.)
    - See more information in [`vc_reflect::access`].

## Examples


### Derive reflection and inspect type info

```rust
use vc_reflect::{derive::Reflect, info::Typed};

#[derive(Reflect)]
struct Player {
    name: String,
    level: u32,
    health: f32,
}

let info = <Player as Typed>::type_info().as_struct().unwrap();

assert_eq!(info.field_len(), 3);
assert_eq!(info.index_of("name"), Some(0));
assert_eq!(info.index_of("health"), Some(2));
```

### Register a type and construct it dynamically

```rust
use core::any::TypeId;
use vc_reflect::{
    derive::Reflect,
    registry::{ReflectDefault, TypeRegistry},
};

#[derive(Reflect, Default)]
#[reflect(default)]
struct Enemy {
    species: String,
    hp: u32,
}

let mut registry = TypeRegistry::default();
registry.register::<Enemy>();

assert!(registry.get_type_info(TypeId::of::<Enemy>()).is_some());

let default_ctor = registry
    .get_type_trait::<ReflectDefault>(TypeId::of::<Enemy>())
    .unwrap();

let enemy = default_ctor.default().take::<Enemy>().unwrap();

assert!(enemy.species.is_empty());
assert_eq!(enemy.hp, 0);
```

### Read nested data with path access

```rust
use vc_reflect::{
    access::{PathAccessor, ReflectPathAccess},
    derive::Reflect,
};

#[derive(Reflect)]
struct Inventory {
    coins: u32,
    slots: Vec<Option<String>>,
}

let inventory = Inventory {
    coins: 42,
    slots: vec![Some("Sword".to_string()), None],
};

assert_eq!(*inventory.access_as::<u32>(".coins").unwrap(), 42);

let accessor = PathAccessor::parse_static(".slots[0]").unwrap();
let first_slot = accessor.access_as::<Option<String>>(&inventory).unwrap();

assert_eq!(first_slot.as_deref(), Some("Sword"));
```

These examples cover the most common entry points:

- derive `Reflect` to expose runtime type information
- use `TypeRegistry` when you need metadata or constructors without holding an instance
- use `access` helpers to inspect nested reflected data from strings or cached paths



## Feature Flags

### `default`

Includes `std` , `debug` and `auto_register`.

### `std`

Enabled by default.

Provide reflection implementations for standard library containers like `HashMap`.

### `debug`

Enabled by default, but only takes effect in debug mode.

When turned on, we will test the validity of the data in many places
and record type information stack during serialization and deserialization.

### `auto_register`

Enabled by default.

Enables automatic type registration through static initialization.

When disabled, auto-registration functions remain available but perform no operation.

See [`TypeRegistry::auto_register`](crate::registry::TypeRegistry::auto_register) for details.

### `reflect_docs`

Enables type documentation collection. Automatically gathers standard documentation
from `#[doc = "..."]` attributes. Disabled by default.

When disabled, documentation functions remain available but always return empty values.

See [`TypeInfo::docs`](crate::info::TypeInfo::docs) for details.

[`Struct`]: ops::Struct
[`Enum`]: ops::Enum
[`Tuple`]: ops::Tuple