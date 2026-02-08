use crate::{
    FromReflect, Reflect,
    derive::impl_type_path,
    info::Typed,
    ops::{ApplyError, List, ListItemIter, ReflectCloneError},
};
use alloc::{boxed::Box, vec::Vec};

use vc_utils::vec::{AutoVec, FastVec, StackVec};

impl_type_path!((in fastvec) StackVec<T, const N: usize>);
impl_type_path!((in fastvec) FastVec<T, const N: usize>);
impl_type_path!((in fastvec) AutoVec<T, const N: usize>);

macro_rules! impl_fast_vec_for {
    ($name:ty) => {
        impl<T: $crate::FromReflect + $crate::info::Typed, const N: usize> $crate::info::Typed
            for $name
        {
            fn type_info() -> &'static $crate::info::TypeInfo {
                static CELL: $crate::impls::GenericTypeInfoCell =
                    $crate::impls::GenericTypeInfoCell::new();
                CELL.get_or_insert::<Self>(|| {
                    $crate::info::TypeInfo::List(
                        $crate::info::ListInfo::new::<Self, T>().with_generics(
                            $crate::info::Generics::from([
                                $crate::info::GenericInfo::from(
                                    $crate::info::TypeParamInfo::new::<T>("T"),
                                ),
                                $crate::info::GenericInfo::from(
                                    $crate::info::ConstParamInfo::new::<usize>("N", N),
                                ),
                            ]),
                        ),
                    )
                })
            }
        }

        impl<
            T: $crate::FromReflect + $crate::info::Typed + $crate::registry::GetTypeMeta,
            const N: usize,
        > $crate::registry::GetTypeMeta for $name
        {
            fn get_type_meta() -> $crate::registry::TypeMeta {
                let mut meta = $crate::registry::TypeMeta::with_capacity::<Self>(3);
                meta.insert_trait::<$crate::registry::TypeTraitFromPtr>(
                    $crate::registry::FromType::<Self>::from_type(),
                );
                meta.insert_trait::<$crate::registry::TypeTraitFromReflect>(
                    $crate::registry::FromType::<Self>::from_type(),
                );
                meta.insert_trait::<$crate::registry::TypeTraitDefault>(
                    $crate::registry::FromType::<Self>::from_type(),
                );
                meta
            }
        }
    };
}

impl_fast_vec_for!(StackVec<T, N>);
impl_fast_vec_for!(AutoVec<T, N>);

impl<T: FromReflect + Typed, const N: usize> FromReflect for StackVec<T, N> {
    fn from_reflect(reflect: &dyn Reflect) -> Option<Self> {
        let ref_list = reflect.reflect_ref().as_list().ok()?;

        if ref_list.len() > N {
            return None;
        }

        let mut new_list = Self::new();

        for field in ref_list.iter() {
            #[expect(unsafe_code, reason = "ref_list.len() <= N.")]
            unsafe {
                new_list.push_unchecked(T::from_reflect(field)?);
            }
        }

        Some(new_list)
    }
}

impl<T: FromReflect + Typed, const N: usize> Reflect for StackVec<T, N> {
    crate::reflection::impl_reflect_cast_fn!(List);

    fn to_dynamic(&self) -> Box<dyn Reflect> {
        Box::new(<Self as List>::to_dynamic_list(self))
    }

    fn apply(&mut self, value: &dyn Reflect) -> Result<(), ApplyError> {
        crate::impls::list_apply(self, value)
    }

    fn reflect_clone(&self) -> Result<Box<dyn Reflect>, ReflectCloneError> {
        let mut vec: Self = Self::new();
        for item in self {
            vec.push(
                item.reflect_clone()?
                    .take()
                    .expect("`Reflect::reflect_clone` should return the same type"),
            );
        }

        Ok(Box::new(vec))
    }

    fn reflect_eq(&self, other: &dyn Reflect) -> Option<bool> {
        crate::impls::list_eq(self, other)
    }

    fn reflect_cmp(&self, value: &dyn Reflect) -> Option<::core::cmp::Ordering> {
        crate::impls::list_cmp(self, value)
    }

    fn reflect_hash(&self) -> Option<u64> {
        crate::impls::list_hash(self)
    }

    fn reflect_debug(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        crate::impls::list_debug(self, f)
    }
}

impl<T: FromReflect + Typed, const N: usize> List for StackVec<T, N> {
    fn get(&self, index: usize) -> Option<&dyn Reflect> {
        <[T]>::get(self, index).map(Reflect::as_reflect)
    }

    fn get_mut(&mut self, index: usize) -> Option<&mut dyn Reflect> {
        <[T]>::get_mut(self, index).map(Reflect::as_reflect_mut)
    }

    fn insert(&mut self, index: usize, element: Box<dyn Reflect>) {
        let element = match T::take_from_reflect(element) {
            Ok(v) => v,
            Err(e) => panic! {
                "incompatible type: from {} to {}",
                e.reflect_type_path(),
                T::type_path(),
            },
        };
        Self::insert(self, index, element);
    }

    fn remove(&mut self, index: usize) -> Box<dyn Reflect> {
        Box::new(Self::remove(self, index))
    }

    fn len(&self) -> usize {
        Self::len(self)
    }

    fn is_empty(&self) -> bool {
        Self::is_empty(self)
    }

    fn push(&mut self, value: Box<dyn Reflect>) {
        let value = match T::take_from_reflect(value) {
            Ok(v) => v,
            Err(e) => panic! {
                "incompatible type: from {} to {}",
                e.reflect_type_path(),
                T::type_path(),
            },
        };
        Self::push(self, value);
    }

    fn try_push(&mut self, value: Box<dyn Reflect>) -> Result<(), Box<dyn Reflect>> {
        if Self::len(self) >= N {
            return Err(value);
        }
        let value = T::take_from_reflect(value)?;
        Self::push(self, value);
        Ok(())
    }

    fn pop(&mut self) -> Option<Box<dyn Reflect>> {
        Self::pop(self).map(Reflect::into_boxed_reflect)
    }

    fn iter(&self) -> ListItemIter<'_> {
        ListItemIter::new(self)
    }

    fn drain(&mut self) -> Vec<Box<dyn Reflect>> {
        Self::drain(self, ..)
            .map(Reflect::into_boxed_reflect)
            .collect()
    }
}

impl<T: FromReflect + Typed, const N: usize> FromReflect for AutoVec<T, N> {
    fn from_reflect(reflect: &dyn Reflect) -> Option<Self> {
        let ref_list = reflect.reflect_ref().as_list().ok()?;

        let mut new_list = Self::with_capacity(ref_list.len());

        for field in ref_list.iter() {
            new_list.push(T::from_reflect(field)?);
        }

        Some(new_list)
    }
}

impl<T: FromReflect + Typed, const N: usize> Reflect for AutoVec<T, N> {
    crate::reflection::impl_reflect_cast_fn!(List);

    fn to_dynamic(&self) -> Box<dyn Reflect> {
        Box::new(<Self as List>::to_dynamic_list(self))
    }

    fn apply(&mut self, value: &dyn Reflect) -> Result<(), ApplyError> {
        crate::impls::list_apply(self, value)
    }

    fn reflect_clone(&self) -> Result<Box<dyn Reflect>, ReflectCloneError> {
        let mut vec: Self = Self::with_capacity(self.len());
        for item in self {
            vec.push(
                item.reflect_clone()?
                    .take()
                    .expect("`Reflect::reflect_clone` should return the same type"),
            );
        }

        Ok(Box::new(vec))
    }

    fn reflect_eq(&self, other: &dyn Reflect) -> Option<bool> {
        crate::impls::list_eq(self, other)
    }

    fn reflect_cmp(&self, value: &dyn Reflect) -> Option<::core::cmp::Ordering> {
        crate::impls::list_cmp(self, value)
    }

    fn reflect_hash(&self) -> Option<u64> {
        crate::impls::list_hash(self)
    }

    fn reflect_debug(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        crate::impls::list_debug(self, f)
    }
}

impl<T: FromReflect + Typed, const N: usize> List for AutoVec<T, N> {
    fn get(&self, index: usize) -> Option<&dyn Reflect> {
        <[T]>::get(self, index).map(Reflect::as_reflect)
    }

    fn get_mut(&mut self, index: usize) -> Option<&mut dyn Reflect> {
        <[T]>::get_mut(self, index).map(Reflect::as_reflect_mut)
    }

    fn insert(&mut self, index: usize, element: Box<dyn Reflect>) {
        let element = match T::take_from_reflect(element) {
            Ok(v) => v,
            Err(e) => panic! {
                "incompatible type: from {} to {}",
                e.reflect_type_path(),
                T::type_path(),
            },
        };
        Self::insert(self, index, element);
    }

    fn remove(&mut self, index: usize) -> Box<dyn Reflect> {
        Box::new(Self::remove(self, index))
    }

    fn len(&self) -> usize {
        Self::len(self)
    }

    fn is_empty(&self) -> bool {
        Self::is_empty(self)
    }

    fn push(&mut self, value: Box<dyn Reflect>) {
        let value = match T::take_from_reflect(value) {
            Ok(v) => v,
            Err(e) => panic! {
                "incompatible type: from {} to {}",
                e.reflect_type_path(),
                T::type_path(),
            },
        };
        Self::push(self, value);
    }

    fn try_push(&mut self, value: Box<dyn Reflect>) -> Result<(), Box<dyn Reflect>> {
        let value = T::take_from_reflect(value)?;
        Self::push(self, value);
        Ok(())
    }

    fn pop(&mut self) -> Option<Box<dyn Reflect>> {
        Self::pop(self).map(Reflect::into_boxed_reflect)
    }

    fn iter(&self) -> ListItemIter<'_> {
        ListItemIter::new(self)
    }

    fn drain(&mut self) -> Vec<Box<dyn Reflect>> {
        Self::drain(self, ..)
            .map(Reflect::into_boxed_reflect)
            .collect()
    }
}
