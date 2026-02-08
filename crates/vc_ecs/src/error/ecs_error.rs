use core::error::Error;

use alloc::boxed::Box;

pub struct ECSError {
    inner: Box<InnerECSError>,
}

struct InnerECSError {
    error: Box<dyn Error + Send + Sync + 'static>,
}
