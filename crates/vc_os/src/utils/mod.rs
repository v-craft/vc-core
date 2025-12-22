//! This module provides low-level synchronization primitives and concurrent data structures
//! built on atomic operations.
//!
//! ## Primitives
//!
//! - [`OnceFlag`] : A lightweight flag ensuring true appears only once. Useful for
//!   one-time initialization patterns.
//! - [`Futex`] : A resource-free spinlock, serving as the most basic synchronization primitive.
//! - [`SpinLock`] : A spinlock similar to Mutex, but threads busy-wait instead of sleeping.
//!
//! ## Concurrent Queues
//!
//! - [`ArrayQueue`] : A bounded queue implementation (from crossbeam-queue) using a fixed-size
//!   circular array. Suitable for producer-consumer patterns with known capacity limits.
//! - [`ListQueue`] : A custom unbounded queue implementation using a block-linked list with
//!   idle block reuse to minimize memory allocation overhead.

// -----------------------------------------------------------------------------
// Modules

mod array_queue;
mod backoff;
mod cache_paded;
mod futex;
mod list_queue;
mod once_flag;
mod spin_lock;

// -----------------------------------------------------------------------------
// Internal API

pub(crate) use backoff::Backoff;
use cache_paded::CachePadded;

// -----------------------------------------------------------------------------
// Exports

pub use array_queue::ArrayQueue;
pub use futex::{Futex, FutexGuard};
pub use list_queue::ListQueue;
pub use once_flag::OnceFlag;
pub use spin_lock::{SpinLock, SpinLockGuard};

// -----------------------------------------------------------------------------
// Utils for test

#[cfg(all(test, feature = "std"))]
#[allow(dead_code, reason = "tests")]
pub(crate) mod tests {
    use core::{any::Any, panic::AssertUnwindSafe, sync::atomic};
    use std::{boxed::Box, panic, thread};

    pub(crate) fn test_unwind_panic<R>(f: impl FnOnce() -> R) -> Result<R, Box<dyn Any + Send>> {
        let prev_hook = panic::take_hook();
        panic::set_hook(Box::new(|_| {}));

        let result = panic::catch_unwind(AssertUnwindSafe(f));

        panic::set_hook(prev_hook);
        result
    }

    pub(crate) fn test_thread_panic<F, T>(f: F) -> Result<T, Box<dyn Any + Send>>
    where
        F: FnOnce() -> T,
        F: Send + 'static,
        T: Send + 'static,
    {
        let prev_hook = panic::take_hook();
        panic::set_hook(Box::new(|_| {}));
        atomic::fence(atomic::Ordering::SeqCst);
        let result = thread::spawn(f).join();
        panic::set_hook(prev_hook);
        result
    }
}
