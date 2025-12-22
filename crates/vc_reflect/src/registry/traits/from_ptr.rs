#![expect(unsafe_code, reason = "Cast pointers to references is unsafe.")]

use core::any::TypeId;
use vc_ptr::{Ptr, PtrMut};

use crate::Reflect;
use crate::info::Typed;
use crate::registry::FromType;

#[derive(Clone)]
pub struct TypeTraitFromPtr {
    ty_id: TypeId,
    from_ptr: unsafe fn(Ptr) -> &dyn Reflect,
    from_ptr_mut: unsafe fn(PtrMut) -> &mut dyn Reflect,
}

impl TypeTraitFromPtr {
    /// Returns the [`TypeId`] that the [`TypeTraitFromPtr`] was constructed for.
    pub fn ty_id(&self) -> TypeId {
        self.ty_id
    }

    /// Convert `Ptr` into `&dyn Reflect`.
    ///
    /// # Safety
    ///
    /// `val` must be a pointer to value of the type that the [`TypeTraitFromPtr`] was constructed for.
    /// This can be verified by checking that the type id returned by [`TypeTraitFromPtr::ty_id`] is the expected one.
    pub unsafe fn as_reflect<'a>(&self, val: Ptr<'a>) -> &'a dyn Reflect {
        // SAFETY: contract uphold by the caller.
        unsafe { (self.from_ptr)(val) }
    }

    /// Convert `PtrMut` into `&mut dyn Reflect`.
    ///
    /// # Safety
    ///
    /// `val` must be a pointer to a value of the type that the [`TypeTraitFromPtr`] was constructed for
    /// This can be verified by checking that the type id returned by [`TypeTraitFromPtr::ty_id`] is the expected one.
    pub unsafe fn as_reflect_mut<'a>(&self, val: PtrMut<'a>) -> &'a mut dyn Reflect {
        // SAFETY: contract uphold by the caller.
        unsafe { (self.from_ptr_mut)(val) }
    }

    /// Get a function pointer to turn a `Ptr` into `&dyn Reflect` for
    /// the type this [`TypeTraitFromPtr`] was constructed for.
    ///
    /// # Safety
    ///
    /// When calling the unsafe function returned by this method you must ensure that:
    /// - The input `Ptr` points to the `Reflect` type this `TypeTraitFromPtr`
    ///   was constructed for.
    pub fn from_ptr(&self) -> unsafe fn(Ptr) -> &dyn Reflect {
        self.from_ptr
    }

    /// Get a function pointer to turn a `PtrMut` into `&mut dyn Reflect` for
    /// the type this [`TypeTraitFromPtr`] was constructed for.
    ///
    /// # Safety
    ///
    /// When calling the unsafe function returned by this method you must ensure that:
    /// - The input `PtrMut` points to the `Reflect` type this `TypeTraitFromPtr`
    ///   was constructed for.
    pub fn from_ptr_mut(&self) -> unsafe fn(PtrMut) -> &mut dyn Reflect {
        self.from_ptr_mut
    }
}

impl<T: Typed + Reflect> FromType<T> for TypeTraitFromPtr {
    fn from_type() -> Self {
        TypeTraitFromPtr {
            ty_id: TypeId::of::<T>(),
            from_ptr: |ptr| {
                // SAFETY: `from_ptr_mut` is either called in `TypeTraitFromPtr::as_reflect`
                // or returned by `TypeTraitFromPtr::from_ptr`, both lay out the invariants
                // required by `deref`
                unsafe { ptr.as_ref::<T>() as &dyn Reflect }
            },
            from_ptr_mut: |ptr| {
                // SAFETY: same as above
                unsafe { ptr.consume::<T>() as &mut dyn Reflect }
                // unsafe { ptr.as_mut_ref::<T>() as &mut dyn Reflect }
            },
        }
    }
}

crate::derive::impl_type_path!(::vc_reflect::registry::TypeTraitFromPtr);
