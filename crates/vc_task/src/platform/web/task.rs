use alloc::fmt;
use alloc::boxed::Box;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use core::panic::{AssertUnwindSafe, UnwindSafe};
use core::any::Any;

// -----------------------------------------------------------------------------
// Task

/// Wraps `async_executor::Task`, a spawned future.
///
/// A [`Task`] can be awaited to retrieve the output of its future.
///
/// Dropping a [`Task`] cancels it, which means its future won't be
/// polled again.
/// 
/// To drop the [`Task`] handle without canceling it, use [`Task::detach()`]
/// instead.
/// 
/// To cancel a task gracefully and wait until it is fully destroyed,
/// use the [`Task::cancel()`] method.
///
/// Note that canceling a task actually wakes it and reschedules one last time.
/// Then, the executor can destroy the task by simply dropping its [`Runnable`]
/// or by invoking [`Runnable::run()`].
///
/// # Examples
///
/// ```ignore
/// use smol::{future, Executor};
/// use std::thread;
///
/// let ex = Executor::new();
///
/// // Spawn a future onto the executor.
/// let task = ex.spawn(async {
///     println!("Hello from a task!");
///     1 + 2
/// });
///
/// // Run an executor thread.
/// thread::spawn(move || future::block_on(ex.run(future::pending::<()>())));
///
/// // Wait for the task's output.
/// assert_eq!(future::block_on(task), 3);
/// ```
/// 
/// [`Runnable`]: async_task::Runnable
/// [`Runnable::run()`]: async_task::Runnable::run
#[must_use = "Tasks are canceled when dropped, use `.detach()` to run them in the background."]
#[repr(transparent)]
pub struct Task<T>(async_channel::Receiver<Result<T, Box<dyn Any + Send>>>);

// Custom constructors for web and non-web platforms
impl<T: 'static> Task<T> {
    /// Creates a new task by passing the given future to the web
    /// runtime as a promise.
    pub(crate) fn wrap_future(future: impl Future<Output = T> + 'static) -> Self {
        use vc_os::exports::wasm_bindgen_futures::spawn_local;

        let (sender, receiver) = async_channel::bounded(1);

        spawn_local(async move {
            // Catch any panics that occur when polling the future so they can
            // be propagated back to the task handle.
            let value = CatchUnwind(AssertUnwindSafe(future)).await;
            let _ = sender.send(value).await;
        });

        Self(receiver)
    }
}

impl<T> Task<T> {
    /// Detaches the task to let it keep running in the background.
    ///
    /// # Platform-Specific Behavior
    ///
    /// When building for the web, this method has no effect.
    #[inline(always)]
    pub fn detach(self) {
        // Tasks are already treated as detached on the web.
    }

    /// Cancels the task and waits for it to stop running.
    ///
    /// Returns the task's output if it was completed just before it got canceled, or [`None`] if
    /// it didn't complete.
    ///
    /// While it's possible to simply drop the [`Task`] to cancel it, this is a cleaner way of
    /// canceling because it also waits for the task to stop running.
    ///
    /// # Platform-Specific Behavior
    ///
    /// Canceling tasks is unsupported on the web, and this is the same as awaiting the task.
    pub async fn cancel(self) -> Option<T> {
        // Await the task and handle any panics.
        match self.0.recv().await {
            Ok(Ok(value)) => Some(value),
            Err(_) => None,
            Ok(Err(panic)) => {
                // drop this to prevent the panic payload from resuming the panic on drop.
                // this also leaks the box but I'm not sure how to avoid that
                core::mem::forget(panic);
                None
            }
        }
    }

    /// Returns `true` if the current task is finished.
    ///
    /// Unlike poll, it doesn't resolve the final value, it just checks if the task has finished.
    /// Note that in a multithreaded environment, this task can be finished immediately after calling this function.
    #[inline]
    pub fn is_finished(&self) -> bool {
        // We treat the task as unfinished until the result is sent over the channel.
        !self.0.is_empty()
    }
}

impl<T> Future for Task<T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // `recv()` returns a future, so we just poll that and hand the result.
        let recv = core::pin::pin!(self.0.recv());
        match recv.poll(cx) {
            Poll::Ready(Ok(Ok(value))) => Poll::Ready(value),
            // NOTE: Propagating the panic here sorta has parity with the async_executor behavior.
            // For those tasks, polling them after a panic returns a `None` which gets `unwrap`ed, so
            // using `resume_unwind` here is essentially keeping the same behavior while adding more information.
            Poll::Ready(Ok(Err(_panic))) => std::panic::resume_unwind(_panic),
            Poll::Ready(Err(_)) => panic!("Polled a task after it finished running"),
            Poll::Pending => Poll::Pending,
        }
    }
}

// All variants of Task<T> are expected to implement Unpin
impl<T> Unpin for Task<T> {}

// Derive doesn't work for macro types, so we have to implement this manually.
impl<T> fmt::Debug for Task<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

// -----------------------------------------------------------------------------
// CatchUnwind

// Utilities for catching unwinds on the web.
struct CatchUnwind<F: UnwindSafe>(F);

impl<F: Future + UnwindSafe> Future for CatchUnwind<F> {
    type Output = Result<F::Output, Box<dyn Any + Send + 'static>>;
    
    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        #[expect(unsafe_code, reason = "project inner pinned here is safe")]
        let inner_future = unsafe {
            let this = self.get_unchecked_mut();
            Pin::new_unchecked(&mut this.0)
        };

        let f = AssertUnwindSafe(|| inner_future.poll(cx));

        let result = std::panic::catch_unwind(f)?;

        result.map(Ok)
    }
}

// -----------------------------------------------------------------------------
// Tests

#[cfg(test)]
mod tests {
    use super::Task;

    #[test]
    fn is_sync_send() {
        fn is_sync<T: Sync>() {}
        is_sync::<Task<()>>();

        fn is_send<T: Send>() {}
        is_send::<Task<()>>();
    }
}
