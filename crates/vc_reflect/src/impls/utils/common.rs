use alloc::borrow::Cow;
use alloc::format;
use core::fmt;
use core::hash::{Hash, Hasher};

use crate::Reflect;
use crate::info::VariantKind;
use crate::ops::{ApplyError, ReflectRef};
use crate::ops::{Array, Enum, List, Map, Set, Struct, Tuple, TupleStruct};

/// A function use for implementing [`Reflect::try_apply`]
///
/// # Rules
///
/// 1. If `other` is not `Array`, return Err.
/// 2. If `self.len` != `other.len`, return Err.
/// 3. Try to apply all items, return Err if apply item failed.
/// 4. return Ok.
///
/// # Example
///
/// ```ignore
///
/// pub struct Foo { /* ... */ }
///
/// impl Array for Foo{ /* ... */ }
/// impl Reflect for Foo {
///     // ...
///     fn try_apply(&self, other: &dyn Reflect) -> Result<(), ApplyError> {
///         array_try_apply(self, other)
///     }
///     // ...
/// }
/// ```
#[inline(never)]
pub fn array_try_apply(x: &mut dyn Array, y: &dyn Reflect) -> Result<(), ApplyError> {
    let y = y.reflect_ref().as_array()?;

    if x.len() != y.len() {
        return Err(ApplyError::DifferentSize {
            from_size: y.len(),
            to_size: x.len(),
        });
    }

    for (idx, y_item) in y.iter().enumerate() {
        let item = x.get_mut(idx).expect("valid index");
        item.try_apply(y_item)?;
    }
    Ok(())
}

/// A function use for implementing [`Reflect::reflect_partial_eq`].
///
/// # Rules
///
/// 1. If `other` is not `Array`, return `Some(false)`.
/// 2. If `self.len` != `other.len`, return `Some(false)`.
/// 3. Call `reflect_partial_eq` for each items,
///    return `None` or `Some(false)` if items return `None` or `Some(false)`.
/// 4. return Ok.
///
/// # Example
///
/// ```ignore
///
/// pub struct Foo { /* ... */ }
///
/// impl Array for Foo{ /* ... */ }
/// impl Reflect for Foo {
///     // ...
///     fn reflect_partial_eq(&self, other: &dyn Reflect) -> Option<bool> {
///         array_partial_eq(self, other)
///     }
///     // ...
/// }
/// ```
#[inline(never)]
pub fn array_partial_eq(x: &dyn Array, y: &dyn Reflect) -> Option<bool> {
    let ReflectRef::Array(y) = y.reflect_ref() else {
        return Some(false);
    };

    if x.len() != y.len() {
        return Some(false);
    }

    for (item, y_item) in x.iter().zip(y.iter()) {
        let result = item.reflect_partial_eq(y_item);
        if result != Some(true) {
            return Some(false);
        }
    }

    Some(true)
}

/// A function use for implementing [`Reflect::reflect_hash`] .
///
/// # Rules
///
/// Try hash all fields, `self.ty_id` and `self.len`.
///
/// This function guarantees a fixed hash result with [`reflect_hasher`](crate::reflect_hasher).
///
/// # Example
///
/// ```ignore
///
/// pub struct Foo { /* ... */ }
///
/// impl Array for Foo{ /* ... */ }
/// impl Reflect for Foo {
///     // ...
///     fn reflect_hash(&self) -> Option<u64> {
///         array_hash(self)
///     }
///     // ...
/// }
/// ```
#[inline(never)]
pub fn array_hash(x: &dyn Array) -> Option<u64> {
    let mut hasher = crate::reflect_hasher();

    for value in x.iter() {
        hasher.write_u64(value.reflect_hash()?);
    }

    x.ty_id().hash(&mut hasher);
    x.len().hash(&mut hasher);

    Some(hasher.finish())
}

/// A function use for implementing [`Reflect::reflect_debug`] .
///
/// # Example
///
/// ```ignore
///
/// pub struct Foo { /* ... */ }
///
/// impl Array for Foo{ /* ... */ }
/// impl Reflect for Foo {
///     // ...
///     fn reflect_debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
///         array_debug(self, f)
///     }
///     // ...
/// }
/// ```
#[inline(never)]
pub fn array_debug(dyn_array: &dyn Array, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    // This function should only be used to impl `Reflect::debug`
    // Non Inline: only be compiled once -> reduce compilation times
    let mut debug = f.debug_list();
    for item in dyn_array.iter() {
        debug.entry(&item as &dyn fmt::Debug);
    }
    debug.finish()
}

/// A function use for implementing [`Reflect::try_apply`]
///
/// # Rules
///
/// 1. If `other` is not `Tuple`, return Err.
/// 2. If `self.field_len` != `other.field_len`, return Err.
/// 3. Try to apply all items, return Err if apply items failed.
/// 4. return Ok.
///
/// # Example
///
/// ```ignore
///
/// pub struct Foo { /* ... */ }
///
/// impl Tuple for Foo{ /* ... */ }
/// impl Reflect for Foo {
///     // ...
///     fn try_apply(&self, other: &dyn Reflect) -> Result<(), ApplyError> {
///         tuple_try_apply(self, other)
///     }
///     // ...
/// }
/// ```
#[inline(never)]
pub fn tuple_try_apply(x: &mut dyn Tuple, y: &dyn Reflect) -> Result<(), ApplyError> {
    let y = y.reflect_ref().as_tuple()?;

    if x.field_len() != y.field_len() {
        return Err(ApplyError::DifferentSize {
            from_size: y.field_len(),
            to_size: x.field_len(),
        });
    }

    for (idx, y_field) in y.iter_fields().enumerate() {
        let field = x.field_mut(idx).expect("valid index");
        field.try_apply(y_field)?;
    }

    Ok(())
}

/// A function use for implementing [`Reflect::reflect_partial_eq`].
///
/// # Rules
///
/// 1. If `other` is not `Tuple`, return `Some(false)`.
/// 2. If `self.len` != `other.len`, return `Some(false)`.
/// 3. Call `reflect_partial_eq` for each items,
///    return `None` or `Some(false)` if items return `None` or `Some(false)`.
/// 4. return Ok.
///
/// # Example
///
/// ```ignore
///
/// pub struct Foo { /* ... */ }
///
/// impl Tuple for Foo{ /* ... */ }
/// impl Reflect for Foo {
///     // ...
///     fn reflect_partial_eq(&self, other: &dyn Reflect) -> Option<bool> {
///         tuple_partial_eq(self, other)
///     }
///     // ...
/// }
/// ```
#[inline(never)]
pub fn tuple_partial_eq(x: &dyn Tuple, y: &dyn Reflect) -> Option<bool> {
    let ReflectRef::Tuple(y) = y.reflect_ref() else {
        return Some(false);
    };

    if x.field_len() != y.field_len() {
        return Some(false);
    }

    for (x_field, y_field) in x.iter_fields().zip(y.iter_fields()) {
        let result = x_field.reflect_partial_eq(y_field);
        if result != Some(true) {
            return result;
        }
    }
    Some(true)
}

/// A function use for implementing [`Reflect::reflect_hash`] .
///
/// # Rules
///
/// Try hash all fields, `self.ty_id` and `self.field_len`.
///
/// This function guarantees a fixed hash result with [`reflect_hasher`](crate::reflect_hasher).
///
/// # Example
///
/// ```ignore
///
/// pub struct Foo { /* ... */ }
///
/// impl Tuple for Foo{ /* ... */ }
/// impl Reflect for Foo {
///     // ...
///     fn reflect_hash(&self) -> Option<u64> {
///         tuple_hash(self)
///     }
///     // ...
/// }
/// ```
#[inline(never)]
pub fn tuple_hash(x: &dyn Tuple) -> Option<u64> {
    let mut hasher = crate::reflect_hasher();

    for field in x.iter_fields() {
        field.reflect_hash()?.hash(&mut hasher);
    }
    x.ty_id().hash(&mut hasher);
    x.field_len().hash(&mut hasher);

    Some(hasher.finish())
}

/// A function use for implementing [`Reflect::reflect_debug`] .
///
/// # Example
///
/// ```ignore
///
/// pub struct Foo { /* ... */ }
///
/// impl Tuple for Foo{ /* ... */ }
/// impl Reflect for Foo {
///     // ...
///     fn reflect_debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
///         tuple_debug(self, f)
///     }
///     // ...
/// }
/// ```
#[inline(never)]
pub fn tuple_debug(dyn_tuple: &dyn Tuple, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let mut debug = f.debug_tuple("");
    for field in dyn_tuple.iter_fields() {
        debug.field(&field as &dyn fmt::Debug);
    }
    debug.finish()
}

/// A function use for implementing [`Reflect::try_apply`] .
///
/// # Rules
///
/// 1. If `other` is not `Struct`, return Err.
/// 2. Call `try_apply` for common fields between `Self` and `Other`.
///    return `Err` if some fields `try_apply` failed.
/// 3. return `Ok`
///
/// Therefore, this function enables `try_apply` between different structs.
/// Only this type and enum allow this.
///
/// # Example
///
/// ```ignore
///
/// pub struct Foo { /* ... */ }
///
/// impl Struct for Foo{ /* ... */ }
/// impl Reflect for Foo {
///     // ...
///     fn try_apply(&self, other: &dyn Reflect) -> Result<(), ApplyError> {
///         struct_try_apply(self, other)
///     }
///     // ...
/// }
/// ```
#[inline(never)]
pub fn struct_try_apply(x: &mut dyn Struct, y: &dyn Reflect) -> Result<(), ApplyError> {
    let y = y.reflect_ref().as_struct()?;

    for (idx, y_field) in y.iter_fields().enumerate() {
        let name = y.name_at(idx).unwrap();
        if let Some(field) = x.field_mut(name) {
            field.try_apply(y_field)?;
        }
    }
    Ok(())
}

/// A function use for implementing [`Reflect::reflect_partial_eq`] .
///
/// # Rules
///
/// 1. If `other` is not `Struct`, return `Some(false)`.
/// 2. If `self.len` != `other.len`, return `Some(false)`.
/// 3. Call `struct_partial_eq` for all fields.
///    Return `Some(false)` if some fields name do not match.
///    Return `None` or `Some(false)` if some fields return `None` or `Some(false)`.
/// 3. return `Some(true)`
///
/// # Example
///
/// ```ignore
///
/// pub struct Foo { /* ... */ }
///
/// impl Struct for Foo{ /* ... */ }
/// impl Reflect for Foo {
///     // ...
///     fn reflect_partial_eq(&self, other: &dyn Reflect) -> Option<bool> {
///         struct_partial_eq(self, other)
///     }
///     // ...
/// }
/// ```
#[inline(never)]
pub fn struct_partial_eq(x: &dyn Struct, y: &dyn Reflect) -> Option<bool> {
    let ReflectRef::Struct(y) = y.reflect_ref() else {
        return Some(false);
    };

    if x.field_len() != y.field_len() {
        return Some(false);
    }

    for (idx, y_field) in y.iter_fields().enumerate() {
        let name = y.name_at(idx).unwrap();
        if let Some(x_field) = x.field(name) {
            let result = x_field.reflect_partial_eq(y_field);
            if result != Some(true) {
                return result;
            }
        } else {
            return Some(false);
        }
    }
    Some(true)
}

/// A function use for implementing [`Reflect::reflect_hash`] .
///
/// # Rules
///
/// Try hash all fields, `self.ty_id` and `self.len`.
///
/// This function guarantees a fixed hash result with [`reflect_hasher`](crate::reflect_hasher).
///
/// # Example
///
/// ```ignore
///
/// pub struct Foo { /* ... */ }
///
/// impl Struct for Foo{ /* ... */ }
/// impl Reflect for Foo {
///     // ...
///     fn reflect_hash(&self) -> Option<u64> {
///         struct_hash(self)
///     }
///     // ...
/// }
/// ```
#[inline(never)]
pub fn struct_hash(x: &dyn Struct) -> Option<u64> {
    let mut hasher = crate::reflect_hasher();

    for item in x.iter_fields() {
        hasher.write_u64(item.reflect_hash()?);
    }

    x.ty_id().hash(&mut hasher);
    x.field_len().hash(&mut hasher);

    Some(hasher.finish())
}

/// A function use for implementing [`Reflect::reflect_debug`] .
///
/// # Example
///
/// ```ignore
///
/// pub struct Foo { /* ... */ }
///
/// impl Struct for Foo{ /* ... */ }
/// impl Reflect for Foo {
///     // ...
///     fn reflect_debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
///         struct_debug(self, f)
///     }
///     // ...
/// }
/// ```
#[inline(never)]
pub fn struct_debug(dyn_struct: &dyn Struct, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let mut debug = f.debug_struct(dyn_struct.reflect_type_path());

    for (index, field) in dyn_struct.iter_fields().enumerate() {
        debug.field(
            dyn_struct.name_at(index).unwrap(),
            &field as &dyn fmt::Debug,
        );
    }
    debug.finish()
}

/// A function use for implementing [`Reflect::try_apply`] .
///
/// # Rules
///
/// 1. If `other` is not `Enum`, return Err.
/// 2. If variant_name mismatched, return Ok(Some(other)),
///    This means that the function cannot handle it.
/// 3. If variant_name matched, but variant_kind mismatched, return Err.
/// 4. try_apply all fields, similer to `struct_try_apply` and `tuple_try_apply`.
/// 5. return `Ok(None)`
///
/// # Example
///
/// ```ignore
/// fn try_apply(&mut self, value: &dyn Reflect) -> Result<(), ApplyError> {
///     if let Some(y) = enum_try_apply(self, value)? {
///         /* ... */
///     }
///     Ok(())
/// }
/// ```
#[inline(never)]
pub fn enum_try_apply<'b>(
    x: &mut dyn Enum,
    y: &'b dyn Reflect,
) -> Result<Option<&'b dyn Enum>, ApplyError> {
    let y = y.reflect_ref().as_enum()?;

    if x.variant_name() == y.variant_name() {
        if x.variant_kind() != y.variant_kind() {
            return Err(ApplyError::MismatchedTypes {
                from_type: Cow::Owned(y.variant_path()),
                to_type: Cow::Owned(x.variant_path()),
            });
        }
        match y.variant_kind() {
            VariantKind::Struct => {
                for y_field in y.iter_fields() {
                    let name = y_field.name().unwrap();
                    if let Some(field) = x.field_mut(name) {
                        field.try_apply(y_field.value())?;
                    }
                }
            }
            VariantKind::Tuple => {
                if x.field_len() != y.field_len() {
                    return Err(ApplyError::DifferentSize {
                        from_size: y.field_len(),
                        to_size: x.field_len(),
                    });
                }
                for (index, y_field) in y.iter_fields().enumerate() {
                    let field = x.field_at_mut(index).expect("valid index");
                    field.try_apply(y_field.value())?;
                }
            }
            VariantKind::Unit => {}
        }
        Ok(None)
    } else {
        Ok(Some(y))
    }
}

/// A function use for implementing [`Reflect::reflect_partial_eq`] .
///
/// # Rules
///
/// 1. If `other` is not `Struct`, return `Some(false)`.
/// 2. Return `Some(false)` if `variant_name`, `variant_kind`, `field_len` mismatched.
/// 3. Compare all fields.
/// 4. Return `Some(true)`.
///
/// # Example
///
/// ```ignore
///
/// pub struct Foo { /* ... */ }
///
/// impl Enum for Foo{ /* ... */ }
/// impl Reflect for Foo {
///     // ...
///     fn reflect_partial_eq(&self, other: &dyn Reflect) -> Option<bool> {
///         enum_partial_eq(self, other)
///     }
///     // ...
/// }
/// ```
#[inline(never)]
pub fn enum_partial_eq(x: &dyn Enum, y: &dyn Reflect) -> Option<bool> {
    let ReflectRef::Enum(y) = y.reflect_ref() else {
        return Some(false);
    };

    if x.variant_name() != y.variant_name() {
        return Some(false);
    }

    if x.variant_kind() != y.variant_kind() {
        return Some(false);
    }

    if x.field_len() != y.field_len() {
        return Some(false);
    }

    match x.variant_kind() {
        VariantKind::Unit => Some(true),
        VariantKind::Tuple => {
            for (x_value, y_value) in x.iter_fields().zip(y.iter_fields()) {
                let result = x_value.value().reflect_partial_eq(y_value.value());
                if result != Some(true) {
                    return result;
                }
            }
            Some(true)
        }
        VariantKind::Struct => {
            for field in x.iter_fields() {
                if let Some(y_field) = y.field(field.name().unwrap()) {
                    let result = field.value().reflect_partial_eq(y_field);
                    if result != Some(true) {
                        return Some(false);
                    }
                } else {
                    return Some(false);
                }
            }
            Some(true)
        }
    }
}

/// A function use for implementing [`Reflect::reflect_hash`] .
///
/// # Rules
///
/// Try hash all fields, `type_id`, `variant_name` and `variant_kind`.
///
/// This function guarantees a fixed hash result with [`reflect_hasher`](crate::reflect_hasher).
///
/// # Example
///
/// ```ignore
///
/// pub struct Foo { /* ... */ }
///
/// impl Enum for Foo{ /* ... */ }
/// impl Reflect for Foo {
///     // ...
///     fn reflect_hash(&self) -> Option<u64> {
///         enum_hash(self)
///     }
///     // ...
/// }
/// ```
#[inline(never)]
pub fn enum_hash(x: &dyn Enum) -> Option<u64> {
    let mut hasher = crate::reflect_hasher();

    for field in x.iter_fields() {
        hasher.write_u64(field.value().reflect_hash()?);
    }

    x.ty_id().hash(&mut hasher);
    x.variant_name().hash(&mut hasher);
    x.variant_kind().hash(&mut hasher);

    Some(hasher.finish())
}

/// A function use for implementing [`Reflect::reflect_debug`] .
///
/// # Example
///
/// ```ignore
///
/// pub struct Foo { /* ... */ }
///
/// impl Enum for Foo{ /* ... */ }
/// impl Reflect for Foo {
///     // ...
///     fn reflect_debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
///         enum_debug(self, f)
///     }
///     // ...
/// }
/// ```
#[inline(never)]
pub fn enum_debug(dyn_enum: &dyn Enum, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match dyn_enum.variant_kind() {
        VariantKind::Unit => f.write_str(dyn_enum.variant_name()),
        VariantKind::Tuple => {
            let mut debug = f.debug_tuple(dyn_enum.variant_name());
            for field in dyn_enum.iter_fields() {
                debug.field(&field.value() as &dyn fmt::Debug);
            }
            debug.finish()
        }
        VariantKind::Struct => {
            let mut debug = f.debug_struct(dyn_enum.variant_name());
            for field in dyn_enum.iter_fields() {
                debug.field(field.name().unwrap(), &field.value() as &dyn fmt::Debug);
            }
            debug.finish()
        }
    }
}

/// A function use for implementing [`Reflect::try_apply`] .
///
/// # Rules
///
/// 1. If `other` is not `List`, return `Err`.
/// 2. `try_apply` all other items to self.
/// 3. if other.len > self.len, the extra items will call `reflect_clone` or `to_dyncmic`.
///    Check cloned items type and try `push` to self, or return `Err`.
/// 4. if other.len < self.len, `pop` extra items.
/// 5. return `Ok`
///
/// # Example
///
/// ```ignore
///
/// pub struct Foo { /* ... */ }
///
/// impl List for Foo{ /* ... */ }
/// impl Reflect for Foo {
///     // ...
///     fn try_apply(&self, other: &dyn Reflect) -> Result<(), ApplyError> {
///         list_try_apply(self, other)
///     }
///     // ...
/// }
/// ```
#[inline(never)]
pub fn list_try_apply(x: &mut dyn List, y: &dyn Reflect) -> Result<(), ApplyError> {
    let y = y.reflect_ref().as_list()?;

    for (idx, y_item) in y.iter().enumerate() {
        if idx < x.len() {
            if let Some(item) = x.get_mut(idx) {
                item.try_apply(y_item)?;
            } else {
                // Get item error.
                return Err(ApplyError::NotSupport {
                    type_path: Cow::Borrowed(x.reflect_type_path()),
                });
            }
        } else {
            let v = if let Ok(v) = y_item.reflect_clone() {
                v
            } else {
                y_item.to_dynamic()
            };

            if let Err(v) = x.try_push(v) {
                return Err(ApplyError::MismatchedTypes {
                    from_type: Cow::Owned(format!("List<{}>", v.reflect_type_path())),
                    to_type: Cow::Borrowed(x.reflect_type_path()),
                });
            }
        }
    }

    while x.len() > y.len() {
        x.pop();
    }

    Ok(())
}

/// A function use for implementing [`Reflect::reflect_partial_eq`] .
///
/// # Rules
///
/// 1. If `other` is not `List`, return `Some(false)`.
/// 2. Return `Some(false)` if `len` mismatched.
/// 3. Compare all fields.
/// 4. Return `Some(true)`.
///
/// # Example
///
/// ```ignore
///
/// pub struct Foo { /* ... */ }
///
/// impl List for Foo{ /* ... */ }
/// impl Reflect for Foo {
///     // ...
///     fn reflect_partial_eq(&self, other: &dyn Reflect) -> Option<bool> {
///         list_partial_eq(self, other)
///     }
///     // ...
/// }
/// ```
#[inline(never)]
pub fn list_partial_eq(x: &dyn List, y: &dyn Reflect) -> Option<bool> {
    let ReflectRef::List(y) = y.reflect_ref() else {
        return Some(false);
    };

    if x.len() != y.len() {
        return Some(false);
    }

    for (x_value, y_value) in x.iter().zip(y.iter()) {
        let result = x_value.reflect_partial_eq(y_value);
        if result != Some(true) {
            return result;
        }
    }

    Some(true)
}

/// A function use for implementing [`Reflect::reflect_hash`] .
///
/// # Rules
///
/// Try hash all fields, `type_id` and `len`.
///
/// This function guarantees a fixed hash result with [`reflect_hasher`](crate::reflect_hasher).
///
/// # Example
///
/// ```ignore
///
/// pub struct Foo { /* ... */ }
///
/// impl List for Foo{ /* ... */ }
/// impl Reflect for Foo {
///     // ...
///     fn reflect_hash(&self) -> Option<u64> {
///         list_hash(self)
///     }
///     // ...
/// }
/// ```
#[inline(never)]
pub fn list_hash(x: &dyn List) -> Option<u64> {
    let mut hasher = crate::reflect_hasher();

    for val in x.iter() {
        hasher.write_u64(val.reflect_hash()?);
    }

    x.ty_id().hash(&mut hasher);
    x.len().hash(&mut hasher);

    Some(hasher.finish())
}

/// A function use for implementing [`Reflect::reflect_debug`] .
///
/// # Example
///
/// ```ignore
///
/// pub struct Foo { /* ... */ }
///
/// impl List for Foo{ /* ... */ }
/// impl Reflect for Foo {
///     // ...
///     fn reflect_debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
///         list_debug(self, f)
///     }
///     // ...
/// }
/// ```
#[inline(never)]
pub fn list_debug(dyn_list: &dyn List, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let mut debug = f.debug_list();
    for item in dyn_list.iter() {
        debug.entry(&item as &dyn fmt::Debug);
    }
    debug.finish()
}

/// A function use for implementing [`Reflect::try_apply`] .
///
/// # Rules
///
/// 1. If `other` is not `Map`, return `Err`.
/// 2. `try_apply` all other items to self, replace or insert.
/// 3. Reduce oneself and only retain the elements in others.
/// 4. return `Ok`
///
/// # Example
///
/// ```ignore
///
/// pub struct Foo { /* ... */ }
///
/// impl Map for Foo{ /* ... */ }
/// impl Reflect for Foo {
///     // ...
///     fn try_apply(&self, other: &dyn Reflect) -> Result<(), ApplyError> {
///         map_try_apply(self, other)
///     }
///     // ...
/// }
/// ```
#[inline(never)]
pub fn map_try_apply(x: &mut dyn Map, y: &dyn Reflect) -> Result<(), ApplyError> {
    let y = y.reflect_ref().as_map()?;

    for (key, y_val) in y.iter() {
        if let Some(x_val) = x.get_mut(key) {
            x_val.try_apply(y_val)?;
        } else {
            let k = if let Ok(k) = key.reflect_clone() {
                k
            } else {
                key.to_dynamic()
            };

            let v = if let Ok(v) = y_val.reflect_clone() {
                v
            } else {
                y_val.to_dynamic()
            };

            if let Err((k, v)) = x.try_insert(k, v) {
                return Err(ApplyError::MismatchedTypes {
                    from_type: Cow::Owned(format!(
                        "Map<{}, {}>",
                        k.reflect_type_path(),
                        v.reflect_type_path()
                    )),
                    to_type: Cow::Borrowed(x.reflect_type_path()),
                });
            }
        }
    }
    x.retain(&mut |key, _| y.get(key).is_some());

    Ok(())
}

/// A function use for implementing [`Reflect::reflect_partial_eq`] .
///
/// # Rules
///
/// 1. If `other` is not `Map`, return `Some(false)`.
/// 2. Return `Some(false)` if `len` mismatched.
/// 3. Compare all key-value pairs.
/// 4. Return `Some(true)`.
///
/// # Example
///
/// ```ignore
///
/// pub struct Foo { /* ... */ }
///
/// impl Map for Foo{ /* ... */ }
/// impl Reflect for Foo {
///     // ...
///     fn reflect_partial_eq(&self, other: &dyn Reflect) -> Option<bool> {
///         map_partial_eq(self, other)
///     }
///     // ...
/// }
/// ```
#[inline(never)]
pub fn map_partial_eq(x: &dyn Map, y: &dyn Reflect) -> Option<bool> {
    let ReflectRef::Map(y) = y.reflect_ref() else {
        return Some(false);
    };

    if x.len() != y.len() {
        return Some(false);
    }

    for (key, val) in x.iter() {
        if let Some(y_val) = y.get(key) {
            let result = val.reflect_partial_eq(y_val);
            if result != Some(true) {
                return result;
            }
        } else {
            return Some(false);
        }
    }

    Some(true)
}

/// A function use for implementing [`Reflect::reflect_hash`] .
///
/// # Rules
///
/// Try hash all fields, `type_id` and `len`.
///
/// This function guarantees a fixed hash result with [`reflect_hasher`](crate::reflect_hasher).
///
/// # Example
///
/// ```ignore
///
/// pub struct Foo { /* ... */ }
///
/// impl Map for Foo{ /* ... */ }
/// impl Reflect for Foo {
///     // ...
///     fn reflect_hash(&self) -> Option<u64> {
///         map_hash(self)
///     }
///     // ...
/// }
/// ```
#[inline(never)]
pub fn map_hash(x: &dyn Map) -> Option<u64> {
    let mut hasher = crate::reflect_hasher();

    for (key, val) in x.iter() {
        hasher.write_u64(key.reflect_hash()?);
        hasher.write_u64(val.reflect_hash()?);
    }

    x.ty_id().hash(&mut hasher);
    x.len().hash(&mut hasher);

    Some(hasher.finish())
}

/// A function use for implementing [`Reflect::reflect_debug`] .
///
/// # Example
///
/// ```ignore
///
/// pub struct Foo { /* ... */ }
///
/// impl Map for Foo{ /* ... */ }
/// impl Reflect for Foo {
///     // ...
///     fn reflect_debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
///         map_debug(self, f)
///     }
///     // ...
/// }
/// ```
#[inline(never)]
pub fn map_debug(dyn_map: &dyn Map, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let mut debug = f.debug_map();
    for (key, value) in dyn_map.iter() {
        debug.entry(&key as &dyn fmt::Debug, &value as &dyn fmt::Debug);
    }
    debug.finish()
}

/// A function use for implementing [`Reflect::try_apply`] .
///
/// # Rules
///
/// 1. If `other` is not `Set`, return `Err`.
/// 2. `try_apply` all other items to self, insert or skip.
/// 3. Reduce oneself and only retain the elements in others.
/// 4. return `Ok`
///
/// # Example
///
/// ```ignore
///
/// pub struct Foo { /* ... */ }
///
/// impl Set for Foo{ /* ... */ }
/// impl Reflect for Foo {
///     // ...
///     fn try_apply(&self, other: &dyn Reflect) -> Result<(), ApplyError> {
///         set_try_apply(self, other)
///     }
///     // ...
/// }
/// ```
#[inline(never)]
pub fn set_try_apply(x: &mut dyn Set, y: &dyn Reflect) -> Result<(), ApplyError> {
    let y = y.reflect_ref().as_set()?;

    for y_val in y.iter() {
        if !x.contains(y_val) {
            let v = if let Ok(v) = y_val.reflect_clone() {
                v
            } else {
                y_val.to_dynamic()
            };
            if let Err(v) = x.try_insert(v) {
                return Err(ApplyError::MismatchedTypes {
                    from_type: Cow::Owned(format!("Set<{}>", v.reflect_type_path())),
                    to_type: Cow::Borrowed(x.reflect_type_path()),
                });
            }
        }
    }
    x.retain(&mut |val| y.contains(val));
    Ok(())
}

/// A function use for implementing [`Reflect::reflect_partial_eq`] .
///
/// # Rules
///
/// 1. If `other` is not `Set`, return `Some(false)`.
/// 2. Return `Some(false)` if `len` mismatched.
/// 3. Compare all values.
/// 4. Return `Some(true)`.
///
/// ```ignore
///
/// pub struct Foo { /* ... */ }
///
/// impl Set for Foo{ /* ... */ }
/// impl Reflect for Foo {
///     // ...
///     fn reflect_partial_eq(&self, other: &dyn Reflect) -> Option<bool> {
///         set_partial_eq(self, other)
///     }
///     // ...
/// }
/// ```
#[inline(never)]
pub fn set_partial_eq(x: &dyn Set, y: &dyn Reflect) -> Option<bool> {
    let ReflectRef::Set(y) = y.reflect_ref() else {
        return Some(false);
    };
    if x.len() != y.len() {
        return Some(false);
    }

    for val in x.iter() {
        if let Some(y_val) = y.get(val) {
            let result = val.reflect_partial_eq(y_val);
            if result != Some(true) {
                return result;
            }
        } else {
            return Some(false);
        }
    }
    Some(true)
}

/// A function use for implementing [`Reflect::reflect_hash`] .
///
/// # Rules
///
/// Try hash all fields, `type_id` and `len`.
///
/// This function guarantees a fixed hash result with [`reflect_hasher`](crate::reflect_hasher).
///
/// # Example
///
/// ```ignore
///
/// pub struct Foo { /* ... */ }
///
/// impl Set for Foo{ /* ... */ }
/// impl Reflect for Foo {
///     // ...
///     fn reflect_hash(&self) -> Option<u64> {
///         set_hash(self)
///     }
///     // ...
/// }
/// ```
#[inline(never)]
pub fn set_hash(x: &dyn Set) -> Option<u64> {
    let mut hasher = crate::reflect_hasher();

    for item in x.iter() {
        hasher.write_u64(item.reflect_hash()?);
    }

    x.ty_id().hash(&mut hasher);
    x.len().hash(&mut hasher);

    Some(hasher.finish())
}

/// A function use for implementing [`Reflect::reflect_debug`] .
///
/// # Example
///
/// ```ignore
///
/// pub struct Foo { /* ... */ }
///
/// impl Set for Foo{ /* ... */ }
/// impl Reflect for Foo {
///     // ...
///     fn reflect_debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
///         set_debug(self, f)
///     }
///     // ...
/// }
/// ```
#[inline(never)]
pub fn set_debug(dyn_set: &dyn Set, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let mut debug = f.debug_set();
    for value in dyn_set.iter() {
        debug.entry(&value as &dyn fmt::Debug);
    }
    debug.finish()
}

/// A function use for implementing [`Reflect::try_apply`] .
///
/// # Rules
///
/// 1. If `other` is not `TupleStruct`, return Err.
/// 2. If `self.field_len` != `other.field_len`, return Err.
/// 3. Try to apply all fields, return Err if apply fields failed.
/// 4. return Ok.
///
/// Therefore, this function enables `try_apply` between different structs.
///
/// # Example
///
/// ```ignore
///
/// pub struct Foo { /* ... */ }
///
/// impl TupleStruct for Foo{ /* ... */ }
/// impl Reflect for Foo {
///     // ...
///     fn try_apply(&self, other: &dyn Reflect) -> Result<(), ApplyError> {
///         tuple_struct_try_apply(self, other)
///     }
///     // ...
/// }
/// ```
#[inline(never)]
pub fn tuple_struct_try_apply(x: &mut dyn TupleStruct, y: &dyn Reflect) -> Result<(), ApplyError> {
    let y = y.reflect_ref().as_tuple_struct()?;

    if x.field_len() != y.field_len() {
        return Err(ApplyError::DifferentSize {
            from_size: y.field_len(),
            to_size: x.field_len(),
        });
    }

    for (idx, y_field) in y.iter_fields().enumerate() {
        let field = x.field_mut(idx).expect("valid index");
        field.try_apply(y_field)?;
    }
    Ok(())
}

/// A function use for implementing [`Reflect::reflect_partial_eq`] .
///
/// # Rules
///
/// 1. If `other` is not `TupleStruct`, return `Some(false)`.
/// 2. Return `Some(false)` if `len` mismatched.
/// 3. Compare all values.
/// 4. Return `Some(true)`.
///
/// ```ignore
///
/// pub struct Foo { /* ... */ }
///
/// impl TupleStruct for Foo{ /* ... */ }
/// impl Reflect for Foo {
///     // ...
///     fn reflect_partial_eq(&self, other: &dyn Reflect) -> Option<bool> {
///         tuple_struct_partial_eq(self, other)
///     }
///     // ...
/// }
/// ```
#[inline(never)]
pub fn tuple_struct_partial_eq(x: &dyn TupleStruct, y: &dyn Reflect) -> Option<bool> {
    let ReflectRef::TupleStruct(y) = y.reflect_ref() else {
        return Some(false);
    };

    if x.field_len() != y.field_len() {
        return Some(false);
    }

    for (x_value, y_value) in x.iter_fields().zip(y.iter_fields()) {
        let result = x_value.reflect_partial_eq(y_value);
        if result != Some(true) {
            return result;
        }
    }

    Some(true)
}

/// A function use for implementing [`Reflect::reflect_hash`] .
///
/// # Rules
///
/// Try hash all fields, `type_id` and `field_len`.
///
/// This function guarantees a fixed hash result with [`reflect_hasher`](crate::reflect_hasher).
///
/// # Example
///
/// ```ignore
///
/// pub struct Foo { /* ... */ }
///
/// impl TupleStruct for Foo{ /* ... */ }
/// impl Reflect for Foo {
///     // ...
///     fn reflect_hash(&self) -> Option<u64> {
///         tuple_struct_hash(self)
///     }
///     // ...
/// }
/// ```
#[inline(never)]
pub fn tuple_struct_hash(x: &dyn TupleStruct) -> Option<u64> {
    let mut hasher = crate::reflect_hasher();

    for item in x.iter_fields() {
        hasher.write_u64(item.reflect_hash()?);
    }

    x.ty_id().hash(&mut hasher);
    x.field_len().hash(&mut hasher);

    Some(hasher.finish())
}

/// A function use for implementing [`Reflect::reflect_debug`] .
///
/// # Example
///
/// ```ignore
///
/// pub struct Foo { /* ... */ }
///
/// impl TupleStruct for Foo{ /* ... */ }
/// impl Reflect for Foo {
///     // ...
///     fn reflect_debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
///         tuple_struct_debug(self, f)
///     }
///     // ...
/// }
/// ```
#[inline(never)]
pub fn tuple_struct_debug(
    dyn_tuple_struct: &dyn TupleStruct,
    f: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    let mut debug = f.debug_tuple(dyn_tuple_struct.reflect_type_path());
    for field in dyn_tuple_struct.iter_fields() {
        debug.field(&field as &dyn fmt::Debug);
    }
    debug.finish()
}
