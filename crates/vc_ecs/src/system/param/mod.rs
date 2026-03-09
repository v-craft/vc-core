//! System parameter infrastructure.
//!
//! Types implementing [`SystemParam`] can appear in a system function signature.
//! The scheduler uses the trait metadata to register state, build access tables,
//! and fetch concrete parameter values for each run.
//!
//! Most users do not implement these traits directly. Instead, they compose the
//! built-in parameters such as [`crate::borrow::Res`], [`crate::borrow::ResMut`],
//! [`crate::borrow::NonSend`], [`Local`], and tuples of those parameters.

// -----------------------------------------------------------------------------
// Modules

mod local;
mod marker;
mod resource;
mod tuples;
mod world;

// -----------------------------------------------------------------------------
// marker

pub use local::Local;
pub use marker::{Exclusive, MainThread, NonSend};

// -----------------------------------------------------------------------------
// SystemParam

use super::AccessTable;
use crate::tick::Tick;
use crate::world::{UnsafeWorld, World};

/// Describes how a type is initialized and fetched as a system parameter.
///
/// Each parameter has a persistent [`State`](SystemParam::State) stored alongside
/// the compiled system. That state is initialized once, contributes borrow
/// information to the system access table, and is then used to fetch the concrete
/// [`Item`](SystemParam::Item) passed into the system body on each run.
///
/// Built-in implementations cover individual parameters, optional parameters, and
/// tuples of parameters. Manual implementations are primarily for extending the
/// ECS runtime with new parameter kinds.
///
/// # Safety
///
/// Implementations must report access patterns accurately from
/// [`mark_access`](SystemParam::mark_access) and must only produce items from
/// [`get_param`](SystemParam::get_param) that are valid for the supplied world and
/// ticks. Incorrect implementations can violate aliasing guarantees enforced by
/// the scheduler.
pub unsafe trait SystemParam: Sized {
    type State: Send + Sync + 'static;
    type Item<'world, 'state>: SystemParam<State = Self::State>;
    const NON_SEND: bool;
    const EXCLUSIVE: bool;

    fn init_state(world: &mut World) -> Self::State;
    fn mark_access(table: &mut AccessTable, state: &Self::State) -> bool;

    /// # Safety
    /// TODO
    unsafe fn get_param<'w, 's>(
        world: UnsafeWorld<'w>,
        state: &'s mut Self::State,
        last_run: Tick,
        this_run: Tick,
    ) -> Self::Item<'w, 's>;
}

/// Marker trait for parameters that only perform shared reads.
///
/// Read-only parameters can participate in systems that run concurrently with
/// other readers of the same data.
///
/// # Safety
/// Ensure by caller.
pub unsafe trait ReadOnlySystemParam: SystemParam {}
