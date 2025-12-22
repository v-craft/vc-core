use core::fmt;
use core::marker::PhantomData;
use core::mem::{self, MaybeUninit};
use core::ptr::{self, NonNull};

/// A `Box`-like pointer for moving a value to a new memory location
/// without needing to pass by value.
///
/// Normally used for partially moving struct fields.
///
/// # Ownership
///
/// This pointer is **not** responsible for freeing the memory pointed to by this pointer
/// as it may be pointing to an element in a `Vec` or to a local in a function etc.
///
/// Conceptually represents ownership of whatever data is being pointed to
/// and will **auto** call its [`Drop`] impl when self be dropped.
/// (Because the type is determined at compile time.)
///
/// # borrow-like
///
/// - It must always point to a valid value of whatever the pointee type is.
/// - The lifetime `'a` accurately represents how long the pointer is valid for.
///
/// # mutable and exclusive
///
/// - It cannot be cloned, and the caller must comply with Rust alias rules.
/// - It does not support pointer arithmetic in any way.
/// - the pointer must always be properly aligned for the type `T`.
#[repr(transparent)]
pub struct MovingPtr<'a, T>(NonNull<T>, PhantomData<&'a mut T>);

impl<T> Drop for MovingPtr<'_, T> {
    fn drop(&mut self) {
        // SAFETY: See `NonNull::drop_in_place`
        unsafe {
            self.0.drop_in_place();
        };
    }
}

impl<'a> crate::OwningPtr<'a> {
    /// Casts to a concrete type as a [`MovingPtr`].
    ///
    /// The caller must ensure the pointer is suitable for `T`.
    /// It is recommended to use [`debug_assert_aligned`](Self::debug_assert_aligned)
    /// to check alignment before calling.
    ///
    /// # Safety
    /// - `T` must be the erased pointee type for this [`OwningPtr`](crate::OwningPtr).
    /// - `ptr` must be properly aligned for type `T`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use vc_ptr::{OwningPtr, MovingPtr};
    /// # use core::mem::ManuallyDrop;
    /// let mut x = ManuallyDrop::new("123".to_string());
    /// let ptr = OwningPtr::from_value(&mut x);
    ///
    /// ptr.debug_assert_aligned::<String>();
    ///
    /// let ptr = unsafe{ ptr.into_moving::<String>() };
    /// ```
    #[inline(always)]
    pub const unsafe fn into_moving<T>(self) -> MovingPtr<'a, T> {
        MovingPtr(self.0.cast::<T>(), PhantomData)
    }
}

impl<'a, T> MovingPtr<'a, T> {
    /// Creates a [`MovingPtr`] from a provided value of type `T`.
    ///
    /// The input value must be initialized because the returned `MovingPtr`
    /// is a pointer to `T`, not `MaybeUninit<T>`.
    ///
    /// The input type is `MaybeUninit<T>` because some fields of the target
    /// are about to be moved through this type.
    ///
    /// # Safety
    /// - `value` must store a properly **initialized** value of type `T`.
    /// - Once the returned [`MovingPtr`] has been used, `value` must be treated as
    ///   it were uninitialized unless it was explicitly leaked via [`core::mem::forget`].
    ///
    /// This is unsafe because we cannot ensure the input is initialized.
    ///
    /// # Examples
    ///
    /// ```
    /// # use vc_ptr::MovingPtr;
    /// # use core::{ptr::NonNull, mem::MaybeUninit};
    /// let mut x = MaybeUninit::new("1234".to_string());
    ///
    /// let ptr = unsafe { MovingPtr::from_value(&mut x) };
    /// ```
    #[inline]
    pub unsafe fn from_value(value: &'a mut MaybeUninit<T>) -> Self {
        // SAFETY:
        // - MaybeUninit<T> has the same memory layout as T
        // - The caller guarantees that `value` must point to a valid instance of type `T`.
        MovingPtr(NonNull::from_mut(value).cast::<T>(), PhantomData)
    }

    /// Creates a new instance from a raw pointer.
    ///
    /// # Safety
    ///
    /// - The data pointed to by this `ptr` must be valid for writes.
    /// - The provided lifetime `'a` must be valid for the pointee.
    /// - `ptr` must point to a valid object of the intended pointee type.
    /// - The data pointed to should use wrapped by [`ManuallyDrop`] or [`MaybeUninit`].
    ///
    /// This function is `unsafe` because callers must uphold the above invariants;
    /// violating them may lead to undefined behavior.
    ///
    /// # Examples
    ///
    /// ```
    /// # use vc_ptr::MovingPtr;
    /// # use core::{ptr::NonNull, mem::ManuallyDrop};
    /// let mut x = ManuallyDrop::new("1234".to_string());
    ///
    /// let ptr = unsafe { MovingPtr::new(NonNull::from_mut(&mut x)) };
    /// ```
    ///
    /// [`ManuallyDrop`]: core::mem::ManuallyDrop
    #[inline]
    pub unsafe fn new(inner: NonNull<T>) -> Self {
        Self(inner, PhantomData)
    }

    /// Consume self and read the value pointed to by this pointer.
    ///
    /// # Examples
    ///
    /// ```
    /// # use vc_ptr::MovingPtr;
    /// # use core::{ptr::NonNull, mem::MaybeUninit};
    /// let mut x = MaybeUninit::new("1234".to_string());
    ///
    /// let ptr = unsafe { MovingPtr::from_value(&mut x) };
    ///
    /// let x = ptr.read();
    /// assert_eq!(x, "1234");
    /// ```
    #[inline]
    pub fn read(self) -> T {
        // SAFETY:
        //  - `self.0` must be valid for reads as this type owns the value it points to.
        //  - `self.0` must always point to a valid instance of type `T`
        //  - `ptr` must be properly aligned for type `T`.
        let value = unsafe { ptr::read(self.0.as_ptr()) };
        mem::forget(self);
        value
    }

    /// Partially moves out some fields inside of `self`.
    ///
    /// The partially returned value is returned back pointing to [`MaybeUninit<T>`].
    ///
    /// While calling this function is safe, care must be taken with the returned `MovingPtr` as it
    /// points to a value that may no longer be completely valid.
    #[inline]
    pub fn partial_move<R>(
        self,
        f: impl FnOnce(MovingPtr<'_, T>) -> R,
    ) -> (MovingPtr<'a, MaybeUninit<T>>, R) {
        let partial_ptr = self.0;
        let ret = f(self);
        (
            MovingPtr(partial_ptr.cast::<MaybeUninit<T>>(), PhantomData),
            ret,
        )
    }

    /// Creates a [`MovingPtr`] for a specific field within `self`.
    ///
    /// This function is explicitly made for deconstructive moves.
    ///
    /// The correct `byte_offset` for a field can be obtained via [`core::mem::offset_of`].
    ///
    /// # Safety
    ///  - `f` must return a non-null pointer to a valid field inside `T`
    ///  - `self` should not be accessed or dropped as if it were a complete value after this function returns.
    ///    Other fields that have not been moved out of may still be accessed or dropped separately.
    ///  - This function cannot alias the field with any other access, including other calls to `move_field`
    ///    for the same field, without first calling [`core::mem::forget`] on it first.
    ///
    /// A result of the above invariants means that any operation that could cause `self` to be dropped while
    /// the pointers to the fields are held will result in undefined behavior. This requires extra caution
    /// around code that may panic. See the example below for an example of how to safely use this function.
    #[inline(always)]
    pub unsafe fn move_field<U>(&self, f: impl Fn(*mut T) -> *mut U) -> MovingPtr<'a, U> {
        MovingPtr(
            // SAFETY: The caller must ensure that `U` is the correct type
            // for the field at `byte_offset`.
            unsafe { NonNull::new_unchecked(f(self.0.as_ptr())) },
            PhantomData,
        )
    }

    /// Writes the value pointed to by this pointer to a provided location.
    ///
    /// This does **not** drop the value stored at `dst` and it's the caller's responsibility
    /// to ensure that it's properly dropped.
    ///
    /// # Safety
    ///  - `dst` must be valid for writes.
    #[inline(always)]
    pub const unsafe fn write_to(self, dst: *mut T) {
        // SAFETY: valid data, aligned ptr
        unsafe { ptr::copy_nonoverlapping(self.0.as_ptr(), dst, 1) };
        mem::forget(self);
    }

    /// Writes the value pointed to by this pointer into `dst`.
    ///
    /// The value previously stored at `dst` will be dropped.
    ///
    /// # Examples
    ///
    /// ```
    /// # use vc_ptr::MovingPtr;
    /// # use core::{ptr::NonNull, mem::MaybeUninit};
    /// let mut x = MaybeUninit::new("1234".to_string());
    ///
    /// let ptr = unsafe { MovingPtr::from_value(&mut x) };
    ///
    /// let mut y = "56789".to_string();
    /// ptr.assign_to(&mut y);
    ///
    /// assert_eq!(y, "1234");
    /// ```
    #[inline(always)]
    pub fn assign_to(self, dst: &mut T) {
        // SAFETY:
        // - `dst` is a mutable borrow,
        // - `dst` must point to a valid instance of `T`.
        // - `dst` must point to value that is valid for dropping.
        // - `dst` must not alias any other access.
        // - `dst` must be valid for writes.
        // - `dst` must always be aligned.
        unsafe {
            ptr::drop_in_place(dst);
            self.write_to(dst);
        }
    }
}

impl<'a, T> MovingPtr<'a, MaybeUninit<T>> {
    /// Creates a [`MovingPtr`] for a specific field within `self`.
    ///
    /// This function is explicitly made for deconstructive moves.
    ///
    /// The correct `byte_offset` for a field can be obtained via [`core::mem::offset_of`].
    ///
    /// # Safety
    ///  - `f` must return a non-null pointer to a valid field inside `T`
    ///  - `self` should not be accessed or dropped as if it were a complete value after this function returns.
    ///    Other fields that have not been moved out of may still be accessed or dropped separately.
    ///  - This function cannot alias the field with any other access, including other calls to [`move_field`]
    ///    for the same field, without first calling [`forget`] on it first.
    ///
    /// [`forget`]: core::mem::forget
    /// [`move_field`]: Self::move_field
    #[inline(always)]
    pub unsafe fn move_maybe_uninit_field<U>(
        &self,
        f: impl Fn(*mut T) -> *mut U,
    ) -> MovingPtr<'a, MaybeUninit<U>> {
        let self_ptr = self.0.as_ptr().cast::<T>();
        // SAFETY:
        // - The caller must ensure that `U` is the correct type for the field at `byte_offset` and thus
        //   cannot be null.
        // - `MaybeUninit<T>` is `repr(transparent)` and thus must have the same memory layout as `T``
        let field_ptr = unsafe { NonNull::new_unchecked(f(self_ptr)) };
        MovingPtr(field_ptr.cast::<MaybeUninit<U>>(), PhantomData)
    }

    /// Creates a [`MovingPtr`] pointing to a valid instance of `T`.
    ///
    /// See also: [`MaybeUninit::assume_init`].
    ///
    /// # Safety
    /// It's up to the caller to ensure that the value pointed to by `self`
    /// is really in an initialized state. Calling this when the content is not yet
    /// fully initialized causes immediate undefined behavior.
    #[inline]
    pub unsafe fn assume_init(self) -> MovingPtr<'a, T> {
        let value = MovingPtr(self.0.cast::<T>(), PhantomData);
        mem::forget(self);
        value
    }
}

impl<T> fmt::Pointer for MovingPtr<'_, T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Pointer::fmt(&self.0, f)
    }
}

impl<T> fmt::Debug for MovingPtr<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MovingPtr({:?})", self.0)
    }
}

impl<T> core::ops::Deref for MovingPtr<'_, T> {
    type Target = T;
    #[inline]
    fn deref(&self) -> &Self::Target {
        // SAFETY: this pointer must be aligned.
        unsafe { &*self.0.as_ptr() }
    }
}

impl<T> core::ops::DerefMut for MovingPtr<'_, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: this pointer must be aligned.
        unsafe { &mut *self.0.as_ptr() }
    }
}

/// Safely converts a owned value into a [`MovingPtr`].
///
/// This cannot be used as expression and must be used as a statement.
///
/// This macro will do two things:
/// 1. Move target to `MaybeUninit<>` in the scope of the macro.
/// 2. Create a MovingPtr with same name.
///
/// # Examples
///
/// ```
/// # use vc_ptr::move_as_ptr;
/// let x = String::from("1234");
/// move_as_ptr!(x);
/// // now `x` is `MovingPtr<String>`
///
/// let y = String::from("1234");
/// move_as_ptr!(y as y_ptr);
/// // now `y_ptr` is `MovingPtr<String>`.
/// // `y` is `MaybeUninit<String>`.
/// ```
#[macro_export]
macro_rules! move_as_ptr {
    ($value:ident) => {
        let mut $value = core::mem::MaybeUninit::new($value);
        // SAFETY: value is initialzed.
        #[expect(unsafe_code, reason = "`MovingPtr::from_value` is unsafe")]
        let $value = unsafe { $crate::MovingPtr::from_value(&mut $value) };
    };
    ($value:ident as $ptr:ident) => {
        let mut $value = core::mem::MaybeUninit::new($value);
        // SAFETY: value is initialzed.
        #[expect(unsafe_code, reason = "`MovingPtr::from_value` is unsafe")]
        let $ptr = unsafe { $crate::MovingPtr::from_value(&mut $value) };
    };
}

/// Helper macro used by [`deconstruct_moving_ptr`]
#[macro_export]
#[doc(hidden)]
macro_rules! get_pattern {
    ($field_index:tt) => {
        $field_index
    };
    ($field_index:tt: $pattern:pat) => {
        $pattern
    };
}

/// Deconstructs a [`MovingPtr`] into its individual fields.
///
/// This consumes the [`MovingPtr`] and hands out [`MovingPtr`] wrappers around
/// pointers to each of its fields. The value will *not* be dropped.
///
/// The macro should wrap a `let` expression with a struct pattern.
/// It does not support matching tuples by position,
/// so for tuple structs you should use `0: pat` syntax.
///
/// For tuples themselves, pass the identifier `tuple` instead of the struct name,
/// like `let tuple { 0: pat0, 1: pat1 } = value`.
///
/// This can also project into `MaybeUninit`.
/// Wrap the type name or `tuple` with `MaybeUninit::<_>`,
/// and the macro will deconstruct a `MovingPtr<MaybeUninit<ParentType>>`
/// into `MovingPtr<MaybeUninit<FieldType>>` values.
///
/// # Examples
///
/// ## Structs
///
/// ```
/// use core::mem::{offset_of, MaybeUninit};
/// use vc_ptr::{MovingPtr, move_as_ptr};
///
/// struct Foo {
///     field_a: i32,
///     field_b: bool,
///     field_c: String,
/// }
///
/// let foo = Foo {
///   field_a: 10,
///   field_b: true,
///   field_c: String::from("hello"),
/// };
///
/// let mut target_a = 5;
/// let mut target_b = false;
/// let mut target_c = String::new();
///
/// // Converts `parent` into a `MovingPtr`
/// move_as_ptr!(foo);
///
/// // The field names must match the name used in the type definition.
/// // Each one will be a `MovingPtr` of the field's type.
/// vc_ptr::deconstruct_moving_ptr!{
///   let Foo { field_a, field_b, field_c } = foo;
/// }
///
/// field_a.assign_to(&mut target_a);
/// field_b.assign_to(&mut target_b);
/// field_c.assign_to(&mut target_c);
///
/// assert_eq!(target_a, 10);
/// assert_eq!(target_b, true);
/// assert_eq!(target_c, "hello");
/// ```
///
/// ## Tuple
///
/// ```
/// use core::mem::{offset_of, MaybeUninit};
/// use vc_ptr::{MovingPtr, move_as_ptr};
///
/// let foo = (10, true, String::from("hello"));
///
/// let mut target_a = 5;
/// let mut target_b = false;
/// let mut target_c = String::new();
///
/// // Converts `parent` into a `MovingPtr`
/// move_as_ptr!(foo);
///
/// // The field names must match the name used in the type definition.
/// // Each one will be a `MovingPtr` of the field's type.
/// vc_ptr::deconstruct_moving_ptr!{
///   let tuple { 0: field_a, 1: field_b, 2: field_c } = foo;
/// }
///
/// field_a.assign_to(&mut target_a);
/// field_b.assign_to(&mut target_b);
/// field_c.assign_to(&mut target_c);
///
/// assert_eq!(target_a, 10);
/// assert_eq!(target_b, true);
/// assert_eq!(target_c, "hello");
/// ```
///
/// ## `MaybeUninit`
///
/// ```
/// use core::mem::{offset_of, MaybeUninit};
/// use vc_ptr::{MovingPtr, move_as_ptr};
///
/// struct Foo {
///     field_a: i32,
///     field_b: bool,
///     field_c: String,
/// }
///
/// let foo = MaybeUninit::new(Foo {
///   field_a: 10,
///   field_b: true,
///   field_c: String::from("hello"),
/// });
///
/// let mut target_a = MaybeUninit::new(5);
/// let mut target_b = MaybeUninit::new(false);
/// let mut target_c = MaybeUninit::new(String::new());
///
/// // Converts `parent` into a `MovingPtr`
/// move_as_ptr!(foo);
///
/// // The field names must match the name used in the type definition.
/// // Each one will be a `MovingPtr` of the field's type.
/// vc_ptr::deconstruct_moving_ptr!{
///   let MaybeUninit::<Foo> { field_a, field_b, field_c } = foo;
/// }
///
/// field_a.assign_to(&mut target_a);
/// field_b.assign_to(&mut target_b);
/// field_c.assign_to(&mut target_c);
///
/// unsafe {
///   assert_eq!(target_a.assume_init(), 10);
///   assert_eq!(target_b.assume_init(), true);
///   assert_eq!(target_c.assume_init(), "hello");
/// }
/// ```
#[macro_export]
macro_rules! deconstruct_moving_ptr {
    { let tuple { $($field_index:tt: $pattern:pat),* $(,)? } = $ptr:expr ;} => {
        let mut ptr: $crate::MovingPtr<_> = $ptr;
        #[cfg(debug_assertions)]
        let _ = || {
            let value = &mut *ptr;
            ::core::hint::black_box(($(&mut value.$field_index,)*));
            fn __unreachable<T>(_index: usize) -> T { unreachable!() }
            *value = ($(__unreachable($field_index),)*);
        };
        // SAFETY: Checked in clousure, see more infomation in `MovingPtr::move_field`.
        $(let $pattern = unsafe { ptr.move_field(|f| &raw mut (*f).$field_index) };)*
        ::core::mem::forget(ptr);
    };
    { let MaybeUninit::<tuple> { $($field_index:tt: $pattern:pat),* $(,)? } = $ptr:expr ;} => {
        let mut ptr: $crate::MovingPtr<::core::mem::MaybeUninit<_> = $ptr;
        #[cfg(debug_assertions)]
        let _ = || {
            let value = unsafe { ptr.assume_init_mut() };
            ::core::hint::black_box(($(&mut value.$field_index,)*));
            fn __unreachable<T>(_index: usize) -> T { unreachable!() }
            *value = ($(__unreachable($field_index),)*);
        };
        // SAFETY: Checked in clousure, see more infomation in `MovingPtr::move_maybe_uninit_field`.
        $(let $pattern = unsafe { ptr.move_maybe_uninit_field(|f| &raw mut (*f).$field_index) };)*
        ::core::mem::forget(ptr);
    };
    { let $struct_name:ident { $($field_index:tt$(: $pattern:pat)?),* $(,)? } = $ptr:expr ;} => {
        let mut ptr: $crate::MovingPtr<_> = $ptr;
        #[cfg(debug_assertions)]
        let _ = || {
            let value = &mut *ptr;
            let $struct_name { $($field_index: _),* } = value;
            ::core::hint::black_box(($(&mut value.$field_index),*));
            let value: *mut _ = value;
            $struct_name { ..unsafe { value.read() } };
        };
        // SAFETY: Checked in clousure, see more infomation in `MovingPtr::move_field`.
        $(let $crate::get_pattern!($field_index$(: $pattern)?) = unsafe { ptr.move_field(|f| &raw mut (*f).$field_index) };)*
        ::core::mem::forget(ptr);
    };
    { let MaybeUninit::<$struct_name:ident> { $($field_index:tt$(: $pattern:pat)?),* $(,)? } = $ptr:expr ;} => {
        let mut ptr: $crate::MovingPtr<core::mem::MaybeUninit<_>> = $ptr;
        #[cfg(debug_assertions)]
        let _ = || {
            let value = unsafe { ptr.assume_init_mut() };
            let $struct_name { $($field_index: _),* } = value;
            ::core::hint::black_box(($(&mut value.$field_index),*));
            let value: *mut _ = value;
            $struct_name { ..unsafe { value.read() } };
        };
        // SAFETY: Checked in clousure, see more infomation in `MovingPtr::move_maybe_uninit_field`.
        $(let $crate::get_pattern!($field_index$(: $pattern)?) = unsafe { ptr.move_maybe_uninit_field(|f| &raw mut (*f).$field_index) };)*
        ::core::mem::forget(ptr);
    };
}
