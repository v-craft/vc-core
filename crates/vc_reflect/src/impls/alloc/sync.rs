// Platforms without atomic pointers are not supported.
crate::derive::impl_reflect_opaque!(::alloc::sync::Arc<T: Send + Sync + ?Sized>(clone));
