// -----------------------------------------------------------------------------
// Modules

mod access;
mod error;
mod function;
mod input;
mod meta;
mod name;
mod param;
mod system;

// -----------------------------------------------------------------------------
// Exports

pub use access::{AccessParam, AccessTable, FilterParam, FilterParamBuilder};
pub use error::UninitSystemError;
pub use function::{FunctionSystem, SystemFunction};
pub use input::{In, InMut, InRef, SystemInput};
pub use meta::{SystemFlags, SystemMeta};
pub use name::SystemName;
pub use param::{Local, ReadOnlySystemParam, SystemParam};
pub use system::{IntoMapSystem, IntoPipeSystem, IntoRunIfSystem};
pub use system::{IntoSystem, MapSystem, PipeSystem, RunIfSystem, System};
