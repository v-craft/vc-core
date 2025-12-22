/// An ergonomic abbreviation for [`Default::default()`] to make initializing structs easier.
///
/// # Example
///
/// ```
/// use vc_utils::default;
///
/// #[derive(Default)]
/// struct Foo {
///   a: usize,
///   b: usize,
///   c: usize,
/// }
///
/// // Normally
/// let foo = Foo {
///   a: 10,
///   ..Default::default()
/// };
/// # let foo = Foo {
/// #   a: 10,
/// #   ..Foo::default()
/// # };
///
/// // New
/// let foo = Foo {
///   a: 10,
///   ..default()
/// };
/// ```
#[inline(always)]
pub fn default<T: Default>() -> T {
    T::default()
}
