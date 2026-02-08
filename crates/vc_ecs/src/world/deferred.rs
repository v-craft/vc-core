use super::UnsafeWorldCell;

// -----------------------------------------------------------------------------
// DeferredWorld

pub struct DeferredWorld<'w> {
    world: UnsafeWorldCell<'w>,
}
