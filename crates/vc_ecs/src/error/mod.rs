mod command;
mod context;
mod ecs_error;
mod handler;

pub use command::{CommandWithEntity, HandleError};
pub use context::ErrorContext;
pub use ecs_error::ECSError;

pub type Result<T = (), E = ECSError> = core::result::Result<T, E>;
