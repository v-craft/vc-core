// use core::mem::MaybeUninit;

// use crate::storage::StorageType;

// use vc_ptr::{MovingPtr, OwningPtr};

// pub trait DynamicBundle: Sized {
//     /// An operation on the entity that happens _after_ inserting this bundle.
//     type Effect;

//     unsafe fn get_components(
//         ptr: MovingPtr<'_, Self>,
//         func: &mut impl FnMut(StorageType, OwningPtr<'_>),
//     );

//     unsafe fn apply_effect(ptr: MovingPtr<'_, MaybeUninit<Self>>, entity: &mut EntityWorldMut);
// }

// pub unsafe trait Bundle: DynamicBundle + Send + Sync + 'static {
//     /// Gets this [`Bundle`]'s component ids, in the order of this bundle's
//     /// [`Component`]s This will register the component if it doesn't exist.
//     #[doc(hidden)]
//     fn component_ids(
//         components: &mut ComponentsRegistrator,
//     ) -> impl Iterator<Item = ComponentId> + use<Self>;

//     /// Return a iterator over this [`Bundle`]'s component ids. This will be [`None`] if the component has not been registered.
//     fn get_component_ids(components: &Components) -> impl Iterator<Item = Option<ComponentId>>;
// }

use vc_ptr::{OwningPtr, MovingPtr};

use crate::storage::StorageType;
use crate::component::{ComponentId, Components, ComponentsRegistrator};

pub trait DynamicBundle: Sized {
    type Effect;

    unsafe fn get_components(
        ptr: MovingPtr<'_, Self>,
        func: &mut impl FnMut(StorageType, OwningPtr<'_>),
    );


}


pub unsafe trait Bundle: DynamicBundle + Send + Sync + 'static {
    fn component_ids(
        components: &mut ComponentsRegistrator,
    ) -> impl Iterator<Item = ComponentId> + use<Self>;

    
    fn get_component_ids(components: &Components) -> impl Iterator<Item = Option<ComponentId>>;
}
