// -----------------------------------------------------------------------------
// Modules

mod access;
mod error;
mod filter;
mod function;
mod input;
mod meta;
mod name;
mod param;
mod system;

// -----------------------------------------------------------------------------
// Exports

pub use error::UninitSystemError;
pub use name::SystemName;

pub use access::AccessTable;
pub use filter::{FilterData, FilterParam, FilterParamBuilder};
pub use function::{FunctionSystem, SystemFunction};
pub use input::{In, InMut, InRef, SystemIn, SystemInput};
pub use meta::{SystemFlags, SystemMeta};
pub use param::{ExclusiveMarker, Local, MainThreadMarker, NonSendMarker};
pub use param::{ReadOnlySystemParam, SystemParam};
pub use system::{IntoMapSystem, IntoPipeSystem, IntoRunIfSystem};
pub use system::{IntoSystem, MapSystem, PipeSystem, RunIfSystem, System};
