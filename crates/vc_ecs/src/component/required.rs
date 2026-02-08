use core::fmt;

use vc_os::sync::Arc;
use vc_ptr::OwningPtr;
use vc_utils::index::SparseIndexMap;

use crate::cfg;
use crate::component::{Component, ComponentsRegistrator};
use crate::component::{ComponentId, Components};
use crate::entity::EntityId;
use crate::storage::{SparseSets, StorageType, Table, TableRow};
use crate::tick::Tick;
use crate::utils::{DebugCheckedUnwrap, DebugLocation};

// -----------------------------------------------------------------------------
// RequiredComponent

type ComponentConstructor =
    Arc<dyn Fn(&mut Table, &mut SparseSets, TableRow, EntityId, Tick, DebugLocation)>;

#[derive(Clone)]
pub struct RequiredComponent {
    constructor: ComponentConstructor,
}

// -----------------------------------------------------------------------------
// RequiredComponents

#[derive(Clone)]
pub struct RequiredComponents {
    /// `direct` includes direct dependencies.
    pub(crate) direct: SparseIndexMap<ComponentId, RequiredComponent>,
    /// `all` contains all dependencies and guarantees that dependent items
    /// are stored first (if A depends on B, then B must appear before A).
    pub(crate) all: SparseIndexMap<ComponentId, RequiredComponent>,
}

// -----------------------------------------------------------------------------
// RequiredComponentsRegistrator

pub struct RequiredComponentsRegistrator<'a, 'w> {
    pub(super) registrator: &'a mut ComponentsRegistrator<'w>,
    pub(super) required_components: &'a mut RequiredComponents,
}

// -----------------------------------------------------------------------------
// RequiredComponentsError

#[non_exhaustive]
pub enum RequiredComponentsError {
    /// The component is already a directly required component for the requiree.
    DuplicateRegistration(ComponentId, ComponentId),
    /// Adding the given requirement would create a cycle.
    CyclicRequirement(ComponentId, ComponentId),
    /// An archetype with the component that requires other components already exists
    ArchetypeExists(ComponentId),
}

impl fmt::Display for RequiredComponentsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateRegistration(x, y) => {
                write!(
                    f,
                    "Component {x:?} already directly requires component {y:?}"
                )
            }
            Self::CyclicRequirement(x, y) => {
                write!(
                    f,
                    "Cyclic requirement found: the requiree component {x:?} is required by the required component {y:?}"
                )
            }
            Self::ArchetypeExists(id) => {
                write!(
                    f,
                    "An archetype with the component {id:?} that requires other components already exists"
                )
            }
        }
    }
}

impl fmt::Debug for RequiredComponentsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl core::error::Error for RequiredComponentsError {}

// -----------------------------------------------------------------------------
// RequiredComponent implementation

impl fmt::Debug for RequiredComponent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("RequiredComponent")
    }
}

// Separate code to reduce compilation workload.
#[inline(never)]
fn table_required(
    table: &mut Table,
    data: OwningPtr<'_>,
    id: ComponentId,
    table_row: TableRow,
    tick: Tick,
    caller: DebugLocation,
) {
    unsafe {
        let raw_index = table.get_raw_index(id).debug_checked_unwrap();
        table.init_component(raw_index, table_row, data, tick, caller);
    }
}

// Separate code to reduce compilation workload.
#[inline(never)]
fn sparse_sets_required(
    sparse_sets: &mut SparseSets,
    data: OwningPtr<'_>,
    id: ComponentId,
    entity_id: EntityId,
    tick: Tick,
    caller: DebugLocation,
) {
    unsafe {
        let raw_index = sparse_sets.get_raw_index(id).debug_checked_unwrap();
        sparse_sets.init_component(raw_index, entity_id, data, tick, caller);
    }
}

impl RequiredComponent {
    pub unsafe fn new<C: Component>(component_id: ComponentId, constructor: fn() -> C) -> Self {
        let func: ComponentConstructor = Arc::new(
            move |table, sparse_sets, table_row, entity_id, change_tick, caller| {
                OwningPtr::make(constructor(), |data| match C::STORAGE_TYPE {
                    StorageType::Table => {
                        table_required(table, data, component_id, table_row, change_tick, caller);
                    }
                    StorageType::SparseSet => {
                        sparse_sets_required(
                            sparse_sets,
                            data,
                            component_id,
                            entity_id,
                            change_tick,
                            caller,
                        );
                    }
                });
            },
        );

        Self { constructor: func }
    }

    #[inline(always)]
    pub unsafe fn initialize(
        &self,
        table: &mut Table,
        sparse_sets: &mut SparseSets,
        table_row: TableRow,
        entity_id: EntityId,
        change_tick: Tick,
        caller: DebugLocation,
    ) {
        (self.constructor)(
            table,
            sparse_sets,
            table_row,
            entity_id,
            change_tick,
            caller,
        );
    }
}

impl fmt::Debug for RequiredComponents {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RequiredComponents")
            .field("direct", &self.direct.keys())
            .field("all", &self.all.keys())
            .finish()
    }
}

impl Default for RequiredComponents {
    #[inline(always)]
    fn default() -> Self {
        const { Self::empty() }
    }
}

impl RequiredComponents {
    #[inline(always)]
    pub const fn empty() -> Self {
        Self {
            direct: SparseIndexMap::new(),
            all: SparseIndexMap::new(),
        }
    }

    /// Iterates the ids of all required components. This includes recursive required components.
    pub fn iter_ids(&self) -> impl Iterator<Item = ComponentId> + '_ {
        self.all.keys().copied()
    }

    #[inline(never)]
    unsafe fn register_inherited_required_components_unchecked(
        all: &mut SparseIndexMap<ComponentId, RequiredComponent>,
        required_id: ComponentId,
        required_component: RequiredComponent,
        components: &Components,
    ) {
        cfg::debug! {
            assert!(required_id.index() < components.infos.len());
        }

        // SAFETY: the caller guarantees that `required_id` is valid in `components`.
        let info = unsafe { components.get_info_unchecked(required_id) };

        if !all.contains_key(&required_id) {
            for (&inherited_id, inherited_required) in &info.required_components.all {
                all.entry(inherited_id)
                    .or_insert_with(|| inherited_required.clone());
            }
        }

        all.insert(required_id, required_component);
    }

    #[inline(never)]
    pub unsafe fn rebuild_inherited_required_components(&mut self, components: &Components) {
        // Clear `all`, we are re-initializing it.
        self.all.clear();

        // We assume that the `all` field of other targets is correct and contains all dependencies.
        // Therefore, we only need to implement a two‑level loop via `direct` and `all`, without recursive lookups.
        for (&required_id, required_component) in &self.direct {
            cfg::debug! {
                assert!(required_id.index() < components.infos.len());
            }

            // SAFETY: the caller guarantees that `required_id` is valid in `components`.
            let info = unsafe { components.get_info_unchecked(required_id) };

            if !self.all.contains_key(&required_id) {
                for (&inherited_id, inherited_required) in &info.required_components.all {
                    self.all
                        .entry(inherited_id)
                        .or_insert_with(|| inherited_required.clone());
                }
            }

            self.all.insert(required_id, required_component.clone());
        }
    }

    #[inline(never)]
    unsafe fn register_dynamic_with(
        &mut self,
        component_id: ComponentId,
        components: &Components,
        constructor: impl FnOnce() -> RequiredComponent,
    ) {
        use vc_utils::index::map::Entry;

        #[cold]
        #[inline(never)]
        fn duplicated_register(component_id: ComponentId) -> ! {
            panic!(
                "Error while registering required component {component_id:?}: already directly required"
            )
        }

        // If already registered as a direct required component then bail.
        let entry = match self.direct.entry(component_id) {
            Entry::Vacant(entry) => entry,
            Entry::Occupied(_) => duplicated_register(component_id),
        };

        // Insert into `direct`.
        let required_component = constructor();
        entry.insert(required_component.clone());

        unsafe {
            Self::register_inherited_required_components_unchecked(
                &mut self.all,
                component_id,
                required_component,
                components,
            );
        }
    }

    #[inline]
    pub unsafe fn register_by_id<C: Component>(
        &mut self,
        component_id: ComponentId,
        components: &Components,
        constructor: fn() -> C,
    ) {
        // SAFETY: the caller guarantees that `component_id` is valid for the type `C`.
        let constructor = || unsafe { RequiredComponent::new(component_id, constructor) };

        // SAFETY:
        // - the caller guarantees that `component_id` is valid in `components`
        // - the caller guarantees all other components were registered in `components`;
        // - constructor is guaranteed to create a valid constructor for the component with id `component_id`.
        unsafe { self.register_dynamic_with(component_id, components, constructor) };
    }

    #[inline]
    unsafe fn register<C: Component>(
        &mut self,
        registrator: &mut ComponentsRegistrator<'_>,
        constructor: fn() -> C,
    ) {
        let id = registrator.register_component::<C>();

        // SAFETY: the caller guarantees that `component_id` is valid for the type `C`.
        let constructor = || unsafe { RequiredComponent::new(id, constructor) };

        // SAFETY:
        // - the caller guarantees all other components were registered in `components`;
        // - constructor is guaranteed to create a valid constructor for the component with id `component_id`.
        unsafe { self.register_dynamic_with(id, registrator.components, constructor) };
    }
}

impl<'a, 'w> RequiredComponentsRegistrator<'a, 'w> {
    #[inline]
    pub fn register_required<C: Component>(&mut self, constructor: fn() -> C) {
        // SAFETY: we internally guarantee that all components in `required_components`
        // are registered in `components`
        unsafe {
            self.required_components
                .register(self.registrator, constructor);
        }
    }

    pub unsafe fn register_required_by_id<C: Component>(
        &mut self,
        component_id: ComponentId,
        constructor: fn() -> C,
    ) {
        // SAFETY:
        // - the caller guarantees `component_id` is a valid component in `components` for `C`;
        // - we internally guarantee all other components in `required_components` are registered in `components`.
        unsafe {
            self.required_components.register_by_id::<C>(
                component_id,
                self.registrator.components,
                constructor,
            );
        }
    }

    pub unsafe fn register_required_dynamic_with(
        &mut self,
        component_id: ComponentId,
        constructor: impl FnOnce() -> RequiredComponent,
    ) {
        // SAFETY:
        // - the caller guarantees `component_id` is valid in `components`;
        // - the caller guarantees `constructor` returns a valid constructor for `component_id`;
        // - we internally guarantee all other components in `required_components` are registered in `components`.
        unsafe {
            self.required_components.register_dynamic_with(
                component_id,
                self.registrator.components,
                constructor,
            );
        }
    }
}
