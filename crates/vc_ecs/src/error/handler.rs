use super::{ECSError, ErrorContext};

pub type ErrorHandler = fn(ECSError, ErrorContext);

pub struct DefaultErrorHandler(pub ErrorHandler);
