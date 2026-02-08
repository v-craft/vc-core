use alloc::string::{String, ToString};
use core::fmt;

use crate::cfg;

const ANONYMOUS_NAME: &str = "_unknown_";

// -----------------------------------------------------------------------------
// DebugName

/// A wrapper type that provides debugging information for ECS (Entity Component System) components.
///
/// This type conditionally includes type name information based on compilation settings:
/// - When `debug_assertions` are enabled or the `debug` feature is active, it stores and displays
///   the actual type name.
/// - Otherwise, it displays a placeholder string indicating debugging is disabled.
///
/// This is useful for debugging ECS-related issues where knowing the concrete type of components
/// or systems is valuable, while allowing the debugging overhead to be compiled out in release builds.
///
/// # Examples
///
/// ```
/// use vc_ecs::utils::DebugName;
///
/// // Create a debug name from a type
/// let name = DebugName::type_name::<String>();
/// assert!(!name.parse().is_empty());
///
/// // Create an anonymous debug name
/// let anonymous = DebugName::anonymous();
/// assert_eq!(anonymous.parse(), "_unknown_");
/// ```
#[derive(Clone, Copy)]
pub struct DebugName {
    #[cfg(any(debug_assertions, feature = "debug"))]
    name: fn() -> &'static str,
}

impl DebugName {
    /// Creates a new `DebugName` that will display the type name of the specified type.
    ///
    /// This uses [`core::any::type_name`] internally to obtain the type's name at compile time.
    /// The type name is only stored when debugging is enabled; otherwise, this operation is a no-op.
    ///
    /// # Type Parameters
    /// * `T` - The type whose name should be captured for debugging purposes.
    ///
    /// # Returns
    /// A `DebugName` instance that will display the type name of `T` when formatted,
    /// or a placeholder if debugging is disabled.
    ///
    /// # Examples
    ///
    /// ```
    /// # use vc_ecs::utils::DebugName;
    /// struct MyComponent;
    /// let name = DebugName::type_name::<MyComponent>();
    /// ```
    #[inline(always)]
    pub const fn type_name<T>() -> Self {
        cfg::debug! {
            if {
                Self { name: ::core::any::type_name::<T> }
            } else {
                Self {}
            }
        }
    }

    /// Creates a new anonymous `DebugName` that always displays `_unknown_`.
    ///
    /// This is useful as a fallback when a type name cannot be determined or when
    /// intentionally hiding the type information.
    ///
    /// # Returns
    /// A `DebugName` instance that will always display `_unknown_` when formatted.
    ///
    /// # Examples
    ///
    /// ```
    /// # use vc_ecs::utils::DebugName;
    /// let anonymous = DebugName::anonymous();
    /// assert_eq!(anonymous.parse(), "_unknown_");
    /// ```
    #[inline(always)]
    pub const fn anonymous() -> Self {
        cfg::debug! {
            if {
                Self {
                    name: || { ANONYMOUS_NAME },
                }
            }
            else {
                Self {}
            }
        }
    }

    /// Converts the debug name to a [`String`].
    ///
    /// This is a convenience method that formats the debug name using its [`Display`](fmt::Display)
    /// implementation and returns the result as an owned string.
    ///
    /// # Returns
    /// A `String` containing the formatted debug name.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use vc_ecs::utils::DebugName;
    /// let name = DebugName::type_name::<String>();
    /// let name_string = name.parse();
    /// ```
    #[inline]
    pub fn parse(&self) -> String {
        ToString::to_string(self)
    }
}

/// Formats a fully-qualified Rust type name into a more readable form for debugging output.
///
/// This function performs intelligent collapsing of type names:
/// - For nested modules, it typically shows only the last segment (the type name itself)
/// - For enum types (heuristically detected by uppercase naming), it retains the enum name
/// - Special characters like `<`, `>`, `,`, etc. are preserved to maintain generic type syntax
///
/// # Arguments
/// * `full_name` - The fully-qualified type name as returned by [`core::any::type_name`]
/// * `f` - The formatter to write the collapsed name to
///
/// # Returns
/// A [`fmt::Result`] indicating success or failure of the formatting operation
///
/// # Note
/// This function is only compiled when debugging is enabled, and is marked `#[inline(never)]`
/// to prevent code bloat from repeated inlining.
#[inline(never)]
#[cfg(any(debug_assertions, feature = "debug"))]
fn debug_fmt(full_name: &str, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    /// Collapses a fully-qualified type name segment to its most readable form.
    ///
    /// # Arguments
    /// * `name` - A segment of a type name (e.g., "core::option::Option")
    ///
    /// # Returns
    /// The collapsed version of the type name segment
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
    /// Formats the debug name for display purposes.
    ///
    /// When debugging is enabled, this will show the collapsed type name.
    /// When debugging is disabled, it will show the anonymous placeholder (`_unknown_`).
    ///
    /// # Arguments
    /// * `f` - The formatter to write the debug name to
    ///
    /// # Returns
    /// A [`fmt::Result`] indicating success or failure
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        cfg::debug! {
            if { debug_fmt((self.name)(), f) }
            else { f.write_str(ANONYMOUS_NAME) }
        }
    }
}

impl fmt::Debug for DebugName {
    /// Formats the debug name using the debug formatter.
    ///
    /// This implementation is identical to [`Display`](fmt::Display) for consistency,
    /// showing the same information regardless of whether `{:?}` or `{}` is used.
    ///
    /// # Arguments
    /// * `f` - The formatter to write the debug name to
    ///
    /// # Returns
    /// A [`fmt::Result`] indicating success or failure
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        cfg::debug! {
            if { debug_fmt((self.name)(), f) }
            else { f.write_str(ANONYMOUS_NAME) }
        }
    }
}
