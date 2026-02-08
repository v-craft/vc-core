#![expect(unsafe_code, reason = "read ptr is unsafe")]

use core::any::TypeId;

use vc_ptr::Ptr;
use vc_reflect::Reflect;
use vc_reflect::registry::{TypeRegistry, TypeTraitFromPtr};

use crate::component::Component;

// -----------------------------------------------------------------------------
// SourceComponent

/// Provides read access to the source component
/// (the component being cloned) in a [`ComponentCloneFn`].
pub struct SourceComponent<'a> {
    ptr: Ptr<'a>,
    type_id: TypeId,
}

impl<'a> SourceComponent<'a> {
    pub(crate) fn new(ptr: Ptr<'a>, type_id: Option<TypeId>) -> Self {
        // This is a hack. The `SourceComponent` itself is an internal type and will not
        // be used as a component. Therefore, its `TypeId` can represent "non-clonable"
        // instead of using `Option<TypeId>`. This reduces the struct size by 8 bytes.
        let type_id = type_id.unwrap_or(TypeId::of::<SourceComponent<'static>>());

        Self { ptr, type_id }
    }

    /// Returns the "raw" pointer to the source component.
    pub fn ptr(&self) -> Ptr<'a> {
        self.ptr
    }

    /// Returns a reference to the component on the source entity.
    pub fn read<C>(&self) -> Option<&C>
    where
        C: Component,
    {
        if TypeId::of::<C>() == self.type_id {
            self.ptr.debug_assert_aligned::<C>();
            unsafe { Some(self.ptr.as_ref::<C>()) }
        } else {
            None
        }
    }

    /// Returns a reference to the component on the source entity
    /// as [`&dyn Reflect`](vc_reflect::Reflect).
    pub fn read_reflect(&self, registry: &TypeRegistry) -> Option<&dyn Reflect> {
        // The `TypeTraitFromPtr` retrieved from the registry by `TypeId` should be type‑correct,
        // unless the user has inserted an incorrect `TypeTraitFromPtr` themselves.
        let from_ptr = registry.get_type_trait::<TypeTraitFromPtr>(self.type_id)?;

        // SAFETY: `TypeTraitFromPtr` get by correct TypeId.
        unsafe { Some(from_ptr.as_reflect(self.ptr)) }
    }
}

// -----------------------------------------------------------------------------
// ComponentCloneFn

use crate::entity::ComponentCloneCtx;

/// Function type that can be used to clone a component of an entity.
pub type ComponentCloneFn = fn(&SourceComponent, &mut ComponentCloneCtx);

// -----------------------------------------------------------------------------
// ComponentCloneBehavior

#[derive(Clone, Debug, Default)]
pub enum ComponentCloneBehavior {
    #[default]
    Default,
    Ignore,
    Custom(ComponentCloneFn),
}

impl ComponentCloneBehavior {}

pub fn component_clone_ignore(_source: &SourceComponent, _ctx: &mut ComponentCloneCtx) {}

pub fn component_clone_via_clone<C: Clone + Component>(
    source: &SourceComponent,
    ctx: &mut ComponentCloneCtx,
) {
    if let Some(component) = source.read::<C>() {
        ctx.write_target_component(component.clone());
    }
}

pub fn component_clone_via_reflect(source: &SourceComponent, ctx: &mut ComponentCloneCtx) {
    let Some(app_registry) = ctx.type_registry().cloned() else {
        return;
    };
    let registry = app_registry.read();
    let Some(source_component_reflect) = source.read_reflect(&registry) else {
        return;
    };
    let component_info = ctx.component_info();
    // checked in read_source_component_reflect
    let type_id = component_info.type_id().unwrap();
}
