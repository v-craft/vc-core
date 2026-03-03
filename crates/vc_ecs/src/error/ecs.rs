use alloc::boxed::Box;
use core::error::Error;
use core::fmt::{Debug, Display};

pub struct ECSError {
    error: Box<dyn Error + Send + Sync + 'static>,
}

impl ECSError {
    /// Attempts to downcast the internal error to the given type.
    pub fn downcast_ref<E: Error + 'static>(&self) -> Option<&E> {
        self.error.downcast_ref::<E>()
    }
}

// NOTE: writing the impl this way gives us From<&str> ... nice!
impl<E: Error + Send + Sync + 'static> From<E> for ECSError {
    #[cold]
    fn from(error: E) -> Self {
        ECSError {
            error: Box::new(error),
        }
    }
}

impl Display for ECSError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "{}", self.error)
    }
}

impl Debug for ECSError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "{:?}", self.error)
    }
}

pub type ECSResult<T> = Result<T, ECSError>;
