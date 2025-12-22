crate::derive::impl_reflect! {
    #[reflect(type_path = "core::option::Option")]
    enum Option<T>{
        None,
        Some(T),
    }
}
