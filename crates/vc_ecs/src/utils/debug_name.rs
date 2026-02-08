use alloc::string::{String, ToString};
use core::fmt;

use crate::cfg;

cfg::debug! {
    if { use alloc::borrow::Cow; }
    else { const DISABLED_NAME: &str = "_"; }
}

// -----------------------------------------------------------------------------
// DebugName

/// Wrapper to help debugging ECS issues.
///
/// - If the `debug` feature is enabled or in `Debug` mode, the name will be used.
/// - If it is disabled, a string mentioning the disabled feature will be us.
#[derive(Clone, PartialEq, Eq)]
pub struct DebugName {
    #[cfg(any(debug_assertions, feature = "debug"))]
    name: Cow<'static, str>,
}

impl DebugName {
    /// Create a new `DebugName` from a type by using its [`core::any::type_name`]
    ///
    /// The value will be ignored if the `debug` feature is not enabled
    #[inline(always)]
    pub fn type_name<T>() -> Self {
        cfg::debug! {
            if {
                let type_name = ::core::any::type_name::<T>();
                Self {
                    name: Cow::Borrowed(type_name)
                }
            }
            else {
                Self {}
            }
        }
    }

    #[inline(always)]
    pub const fn anonymous() -> Self {
        cfg::debug! {
            if {
                Self {
                    name: Cow::Borrowed("_unknown_")
                }
            }
            else {
                Self {}
            }
        }
    }

    #[inline]
    pub fn parse(&self) -> String {
        ToString::to_string(&self)
    }
}

impl From<Option<DebugName>> for DebugName {
    #[inline(always)]
    fn from(value: Option<DebugName>) -> Self {
        if let Some(name) = value {
            name
        } else {
            DebugName::anonymous()
        }
    }
}

#[inline(never)]
#[cfg(any(debug_assertions, feature = "debug"))]
fn debug_fmt(full_name: &str, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    fn collapse_type_name(name: &str) -> &str {
        let mut segments = name.rsplit("::");
        let last = segments.next().unwrap();

        // Enums types are retained.
        // As heuristic, we assume the enum type to be uppercase.
        if let Some(second_last) = segments.next()
            && second_last.starts_with(char::is_uppercase)
        {
            let index = name.len() - last.len() - second_last.len() - 2;
            &name[index..]
        } else {
            last
        }
    }

    const SPECIAL_CHARS: [char; 9] = [' ', '<', '>', '(', ')', '[', ']', ',', ';'];

    let mut rest = full_name;

    while !rest.is_empty() {
        let index = rest.find(|c| SPECIAL_CHARS.contains(&c));

        if let Some(index) = index {
            f.write_str(collapse_type_name(&rest[0..index]))?;

            let special = &rest[index..=index];
            f.write_str(special)?;

            rest = &rest[(index + 1)..];
        } else {
            // If there are no special characters left, we're done!
            f.write_str(collapse_type_name(rest))?;
            return Ok(());
        }
    }

    Ok(())
}

impl fmt::Display for DebugName {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        cfg::debug! {
            if { debug_fmt(self.name.as_ref(), f) }
            else { f.write_str(DISABLED_NAME) }
        }
    }
}

impl fmt::Debug for DebugName {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        cfg::debug! {
            if { debug_fmt(self.name.as_ref(), f) }
            else { f.write_str(DISABLED_NAME) }
        }
    }
}
