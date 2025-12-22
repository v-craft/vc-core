use crate::info::Typed;

/// Trait used to generate [`TypeTrait`] for trait reflection.
///
/// This is used by the `#[derive(Reflect)]` macro to generate an implementation
/// of [`TypeTrait`] to pass to [`TypeMeta::insert_trait`].
///
/// # Example
///
/// ```
/// # use vc_reflect::registry::{TypeMeta, TypeTraitDefault, FromType};
/// let mut meta = TypeMeta::of::<String>();
///
/// meta.insert_trait::<TypeTraitDefault>(FromType::<String>::from_type());
/// ```
///
/// [`TypeTrait`]: crate::registry::TypeTrait
/// [`TypeMeta::insert_trait`]: crate::registry::TypeMeta::insert_trait
pub trait FromType<T: Typed> {
    fn from_type() -> Self;
}
