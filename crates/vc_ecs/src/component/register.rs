use alloc::boxed::Box;
use alloc::vec::Vec;

use vc_os::sync::PoisonError;
use vc_utils::extra::TypeIdMap;

use crate::cfg;
use crate::component::{ComponentInfo, RequiredComponents};
use crate::resource::Resource;
use crate::utils::DebugCheckedUnwrap;

use super::{ComponentDescriptor, ComponentId, ComponentIdAllocator, Components};

// -----------------------------------------------------------------------------
// ComponentsRegistrator

#[derive(Debug)]
pub struct ComponentsRegistrator<'w> {
    pub components: &'w mut Components,
    pub allocator: &'w mut ComponentIdAllocator,
    pub check_stack: Vec<ComponentId>,
}

// -----------------------------------------------------------------------------
// ComponentsQueuedRegistrator

#[derive(Debug, Clone, Copy)]
pub struct QueuedRegistrator<'w> {
    components: &'w Components,
    allocator: &'w ComponentIdAllocator,
}

// -----------------------------------------------------------------------------
// QueuedRegistration

pub struct QueuedRegistration {
    registrator: Box<dyn FnOnce(&mut ComponentsRegistrator, ComponentId, ComponentDescriptor)>,
    pub(super) component_id: ComponentId,
    pub(super) descriptor: ComponentDescriptor,
}

// -----------------------------------------------------------------------------
// ComponentsQueuedRegistrator

#[derive(Debug)]
pub struct QueuedComponents {
    pub components: TypeIdMap<QueuedRegistration>,
    pub resources: TypeIdMap<QueuedRegistration>,
    pub dynamic_registrations: Vec<QueuedRegistration>,
}

// -----------------------------------------------------------------------------
// QueuedRegistration Implementation

impl core::fmt::Debug for QueuedRegistration {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("")
            .field(&self.component_id)
            .field(&self.descriptor.debug_name())
            .finish_non_exhaustive()
    }
}

impl QueuedRegistration {
    /// Performs the registration, returning the now valid [`ComponentId`].
    #[inline(always)]
    pub fn register(self, registrator: &mut ComponentsRegistrator) -> ComponentId {
        (self.registrator)(registrator, self.component_id, self.descriptor);
        self.component_id
    }
}

// -----------------------------------------------------------------------------
// ComponentsRegistrator Implementation

use crate::component::Component;
use core::any::{Any, TypeId};

impl<'w> ComponentsRegistrator<'w> {
    /// Constructs a new [`ComponentsRegistrator`].
    ///
    /// # Safety
    ///
    /// The [`Components`] and [`ComponentIdAllocator`] must match.
    pub unsafe fn new(
        components: &'w mut Components,
        allocator: &'w mut ComponentIdAllocator,
    ) -> Self {
        Self {
            components,
            allocator,
            check_stack: Vec::new(),
        }
    }

    /// Converts this [`ComponentsRegistrator`] into a [`QueuedRegistrator`].
    pub fn as_queued(&self) -> QueuedRegistrator<'_> {
        QueuedRegistrator {
            components: self.components,
            allocator: self.allocator,
        }
    }

    pub fn apply_queued_registrations(&mut self) {
        if !self.components.any_queued_mut() {
            return;
        }

        // We must process them one by one.
        //
        // If we were to take all elements out of the queue at once and then
        // register them individually, a component’s dependencies might already
        // exist in the queue but have been taken out prematurely, causing new
        // component IDs to be incorrectly assigned during registration.
        while let Some((_, registration)) = self.components.get_queue_mut().components.remove_one()
        {
            registration.register(self);
        }

        while let Some((_, registration)) = self.components.get_queue_mut().resources.remove_one() {
            registration.register(self);
        }

        // Dynamic components can be taken out all at once, because we do not
        // check for dependencies among them — these are manually registered.
        let dynamics = core::mem::take(&mut self.components.get_queue_mut().dynamic_registrations);

        for registransion in dynamics {
            registransion.register(self);
        }
    }

    /// # Note
    ///
    /// If this method is called multiple times with identical descriptors, a distinct [`ComponentId`]
    /// will be created for each one.
    #[inline]
    pub fn register_dynamic(&mut self, descriptor: ComponentDescriptor) -> ComponentId {
        let id = self.allocator.next_mut();
        // SAFETY: The id is fresh.
        unsafe {
            self.components.register_dynamic(id, descriptor);
        }
        id
    }

    #[inline]
    pub fn register_component<T: Component>(&mut self) -> ComponentId {
        // Return directly if already registered.
        if let Some(id) = self.check_registered_component(TypeId::of::<T>()) {
            return id;
        }

        let component_id = self.allocator.next_mut();

        unsafe {
            self.register_component_unchecked::<T>(component_id);
        }

        component_id
    }

    /// Separate to speed up compilation. Little performance loss of registration is acceptable.
    #[inline(never)]
    fn check_registered_component(&mut self, type_id: TypeId) -> Option<ComponentId> {
        #[cold]
        #[inline(never)]
        fn handle_recursion_error(
            components: &Components,
            check_stack: &[ComponentId],
            required: ComponentId,
        ) -> ! {
            cfg::debug! {
                if {
                    // We don't care about the performance during panic.
                    let recursion_stack = check_stack.iter().map(|id| {
                        components.get_debug_name(*id).parse()
                    }).collect::<Vec<_>>().join("\n → ");

                    let helper = if required == *check_stack.last().unwrap() {
                        alloc::format!("Remove require({}).", components.get_debug_name(required))
                    } else {
                        "If this is intentional, consider merging the components.".into()
                    };
                    panic!("Recursive required components detected: \n{recursion_stack}\nhelp: {helper}")
                } else {
                    panic!("Recursive required components.")
                }
            }
        }

        if let Some(&required) = self.components.component_indices.get(&type_id) {
            // Already registered, check recursion.
            // SAFETY: `ComponentId` is transparent for `NonZeroU32`.
            unsafe {
                use ::core::mem;

                // A hack, because `u32::contains` has SIMD optimization provided
                // by the standard library. See in `core::cmp::SliceContains`.
                let stack_slice =
                    mem::transmute::<&[ComponentId], &[u32]>(self.check_stack.as_slice());

                if stack_slice.contains(mem::transmute::<&ComponentId, &u32>(&required)) {
                    handle_recursion_error(self.components, &self.check_stack, required);
                }
            }

            return Some(required);
        }

        if let Some(registrator) = self.components.get_queue_mut().components.remove(&type_id) {
            // Remove the duplicate items from the queue to avoid repeated registrations.
            return Some(registrator.register(self));
        }

        None
    }

    /// Separate to speed up compilation. Little performance loss of registration is acceptable.
    #[inline]
    unsafe fn register_component_unchecked<T: Component>(&mut self, component_id: ComponentId) {
        use super::RequiredComponentsRegistrator as RCG;

        // allocate id -> register_component -> push id to recursion_check_stack
        self.register_component_and_set_stack(
            TypeId::of::<T>(),
            component_id,
            ComponentDescriptor::new_component::<T>(),
        );

        let mut required_components = const { RequiredComponents::empty() };

        T::register_required_components(
            component_id,
            &mut RCG {
                registrator: self,
                required_components: &mut required_components,
            },
        );

        let info = self.register_required_by_and_pop_stack(component_id, required_components);

        info.hooks.update_from_component::<T>();
    }

    /// Separate to speed up compilation. Little performance loss of registration is acceptable.
    #[inline(never)]
    fn register_component_and_set_stack(
        &mut self,
        type_id: TypeId,
        component_id: ComponentId,
        descriptor: ComponentDescriptor,
    ) -> ComponentId {
        unsafe {
            self.components
                .register_component(type_id, component_id, descriptor);
        }

        self.check_stack.push(component_id);

        component_id
    }

    /// Separate to speed up compilation. Little performance loss of registration is acceptable.
    #[inline(never)]
    fn register_required_by_and_pop_stack(
        &mut self,
        component_id: ComponentId,
        required_components: RequiredComponents,
    ) -> &mut ComponentInfo {
        unsafe {
            self.components
                .register_required_by(component_id, &required_components);
        }

        self.check_stack.pop();

        // Safety: already registered by `Components::register_component`.
        let info = unsafe {
            self.components
                .infos
                .get_unchecked_mut(component_id.index())
                .as_mut()
                .debug_checked_unwrap()
        };

        info.required_components = required_components;

        info
    }

    #[inline]
    pub fn register_resource<T: Resource>(&mut self) -> ComponentId {
        if let Some(id) = self.check_registered_resource(TypeId::of::<T>()) {
            return id;
        }

        let id = self.allocator.next_mut();

        // SAFETY: The resource is not currently registered, the id is fresh,
        // and the `ComponentDescriptor` matches the `TypeId`
        unsafe {
            self.components.register_resource(
                TypeId::of::<T>(),
                id,
                ComponentDescriptor::new_resource::<T>(),
            );
        }

        id
    }

    /// Registers a [non-send resource](crate::system::NonSend) of type `T` with this instance.
    /// If a resource of this type has already been registered, this will return
    /// the ID of the pre-existing resource.
    #[inline]
    pub fn register_non_send<T: Any>(&mut self) -> ComponentId {
        if let Some(id) = self.check_registered_resource(TypeId::of::<T>()) {
            return id;
        }

        let id = self.allocator.next_mut();

        // SAFETY: The resource is not currently registered, the id is fresh,
        // and the `ComponentDescriptor` matches the `TypeId`
        unsafe {
            self.components.register_resource(
                TypeId::of::<T>(),
                id,
                ComponentDescriptor::new_non_send::<T>(),
            );
        }

        id
    }

    /// Separate to speed up compilation. Little performance loss of registration is acceptable.
    #[inline(never)]
    fn check_registered_resource(&mut self, type_id: TypeId) -> Option<ComponentId> {
        if let Some(&id) = self.components.resource_indices.get(&type_id) {
            return Some(id);
        }

        if let Some(registrator) = self.components.get_queue_mut().resources.remove(&type_id) {
            // Remove the duplicate items from the queue to avoid repeated registrations.
            return Some(registrator.register(self));
        }

        None
    }

    /// Equivalent of `Components::any_queued_mut`
    #[inline]
    pub fn any_queued_mut(&mut self) -> bool {
        self.components.any_queued_mut()
    }

    /// Equivalent of `Components::any_queued_mut`
    #[inline]
    pub fn num_queued_mut(&mut self) -> usize {
        self.components.num_queued_mut()
    }
}

impl<'w> QueuedRegistrator<'w> {
    /// Constructs a new [`QueuedRegistrator`].
    #[inline(always)]
    pub fn new(components: &'w Components, allocator: &'w ComponentIdAllocator) -> Self {
        Self {
            components,
            allocator,
        }
    }

    /// Separate to speed up compilation. Little performance loss of registration is acceptable.
    #[inline(never)]
    fn register_arbitrary_component(
        &self,
        type_id: TypeId,
        descriptor: ComponentDescriptor,
        func: Box<dyn FnOnce(&mut ComponentsRegistrator, ComponentId, ComponentDescriptor)>,
    ) -> ComponentId {
        self.components
            .queued
            .write()
            .unwrap_or_else(PoisonError::into_inner)
            .components
            .get_or_insert(type_id, move || QueuedRegistration {
                registrator: func,
                component_id: self.allocator.next(),
                descriptor,
            })
            .component_id
    }

    /// Separate to speed up compilation. Little performance loss of registration is acceptable.
    #[inline(never)]
    fn register_arbitrary_resource(
        &self,
        type_id: TypeId,
        descriptor: ComponentDescriptor,
        func: Box<dyn FnOnce(&mut ComponentsRegistrator, ComponentId, ComponentDescriptor)>,
    ) -> ComponentId {
        self.components
            .queued
            .write()
            .unwrap_or_else(PoisonError::into_inner)
            .resources
            .get_or_insert(type_id, move || QueuedRegistration {
                registrator: func,
                component_id: self.allocator.next(),
                descriptor,
            })
            .component_id
    }

    /// Separate to speed up compilation. Little performance loss of registration is acceptable.
    #[inline(never)]
    fn register_arbitrary_dynamic(
        &self,
        descriptor: ComponentDescriptor,
        func: Box<dyn FnOnce(&mut ComponentsRegistrator, ComponentId, ComponentDescriptor)>,
    ) -> ComponentId {
        let component_id = self.allocator.next();

        self.components
            .queued
            .write()
            .unwrap_or_else(PoisonError::into_inner)
            .dynamic_registrations
            .push(QueuedRegistration {
                registrator: func,
                component_id,
                descriptor,
            });
        component_id
    }

    #[inline]
    pub fn queue_register_component<T: Component>(&self) -> ComponentId {
        self.components
            .get_component_id(TypeId::of::<T>())
            .unwrap_or_else(|| {
                self.register_arbitrary_component(
                    TypeId::of::<T>(),
                    ComponentDescriptor::new_component::<T>(),
                    Box::new(|registrator, id, _| unsafe {
                        registrator.register_component_unchecked::<T>(id)
                    }),
                )
            })
    }

    /// Separate to speed up compilation. Little performance loss of registration is acceptable.
    #[inline(never)]
    fn register_resource_closure(
        type_id: TypeId,
    ) -> Box<dyn FnOnce(&mut ComponentsRegistrator, ComponentId, ComponentDescriptor)> {
        Box::new(move |registrator, id, descriptor| unsafe {
            registrator
                .components
                .register_resource(type_id, id, descriptor)
        })
    }

    #[inline]
    pub fn queue_register_resource<T: Resource>(&self) -> ComponentId {
        self.components
            .get_resource_id(TypeId::of::<T>())
            .unwrap_or_else(|| {
                self.register_arbitrary_resource(
                    TypeId::of::<T>(),
                    ComponentDescriptor::new_resource::<T>(),
                    Self::register_resource_closure(TypeId::of::<T>()),
                )
            })
    }

    #[inline]
    pub fn queue_register_non_send<T: Any>(&self) -> ComponentId {
        self.components
            .get_resource_id(TypeId::of::<T>())
            .unwrap_or_else(|| {
                self.register_arbitrary_resource(
                    TypeId::of::<T>(),
                    ComponentDescriptor::new_non_send::<T>(),
                    Self::register_resource_closure(TypeId::of::<T>()),
                )
            })
    }

    /// # Note
    ///
    /// Technically speaking, the returned [`ComponentId`] is not valid,
    /// but it will become valid later.
    #[inline]
    pub fn queue_register_dynamic(&self, descriptor: ComponentDescriptor) -> ComponentId {
        self.register_arbitrary_dynamic(
            descriptor,
            Box::new(|registrator, id, descriptor| unsafe {
                registrator.components.register_dynamic(id, descriptor);
            }),
        )
    }
}

impl QueuedComponents {
    pub const fn empty() -> Self {
        Self {
            components: TypeIdMap::new(),
            resources: TypeIdMap::new(),
            dynamic_registrations: Vec::new(),
        }
    }

    pub fn find_by_id(&self, id: ComponentId) -> Option<&QueuedRegistration> {
        self.components
            .values()
            .chain(self.resources.values())
            .chain(self.dynamic_registrations.iter())
            .find(|queued| queued.component_id == id)
    }
}
