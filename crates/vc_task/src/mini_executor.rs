//! Alternative to `async_executor` based on [`edge_executor`] by Ivan Markov.
//!
//! [`async_executor`]: https://github.com/smol-rs/async-executor
//! [`edge_executor`]: https://github.com/ivmarkov/edge-executor

#![expect(unsafe_code, reason = "original implementation relies on unsafe")]

use alloc::rc::Rc;
use core::{
    cell::UnsafeCell,
    future::{Future, poll_fn},
    marker::PhantomData,
    task::{Context, Poll},
};

use async_task::{Runnable, Task};
use atomic_waker::AtomicWaker;
use futures_lite::FutureExt;

use vc_os::sync::{Arc, LazyLock};
use vc_os::utils::ListQueue;

struct State {
    queue: ListQueue<Runnable>,
    waker: AtomicWaker,
}

impl State {
    #[inline]
    fn new() -> Self {
        Self {
            queue: ListQueue::with_limit(16),
            waker: AtomicWaker::new(),
        }
    }
}

/// An async executor.
pub struct Executor<'a> {
    state: LazyLock<Arc<State>>,
    _invariant: PhantomData<UnsafeCell<&'a ()>>,
}

impl Default for Executor<'_> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Executor<'a> {
    /// Creates a new executor.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use vc_task::Executor;
    ///
    /// let ex: Executor = Executor::new();
    /// ```
    pub const fn new() -> Self {
        Self {
            state: LazyLock::new(|| Arc::new(State::new())),
            _invariant: PhantomData,
        }
    }

    /// Creates a new task without [`Send`], [`Sync`], and `'static` bounds.
    ///
    /// This function is same as [`spawn()`], except it does not require [`Send`], [`Sync`], and
    /// `'static` on `future`.
    ///
    /// # Safety
    ///
    /// - If `future` is not [`Send`], its [`Runnable`] must be used and dropped on the original
    ///   thread.
    /// - If `future` is not `'static`, borrowed variables must outlive its [`Runnable`].
    ///
    /// [`spawn()`]: Self::spawn
    /// [`Waker`]: core::task::Waker
    unsafe fn spawn_unchecked<F: Future>(&self, fut: F) -> Task<F::Output> {
        let state: Arc<State> = self.state.clone();

        let schedule = move |runnable| {
            state.queue.push(runnable);
            // If `register` has not been called yet, then this does nothing.
            state.waker.wake();
        };

        // SAFETY:
        // - If `future` is not [`Send`], its [`Runnable`] must be used and dropped on the original thread.
        // - If `future` is not `'static`, borrowed variables must outlive its [`Runnable`].
        // - `schedule` is `Send`, `Sync` and `'static`
        let (runnable, task) = unsafe { async_task::spawn_unchecked(fut, schedule) };

        runnable.schedule();

        task
    }

    /// Spawns a task onto the executor.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use edge_executor::Executor;
    ///
    /// let ex: Executor = Default::default();
    ///
    /// let task = ex.spawn(async {
    ///     println!("Hello world");
    /// });
    /// ```
    ///
    /// Note that if the executor's queue size is equal to the number of currently
    /// spawned and running tasks, spawning this additional task might cause the executor to panic
    /// later, when the task is scheduled for polling.
    #[inline]
    pub fn spawn<F>(&self, fut: F) -> Task<F::Output>
    where
        F: Future + Send + 'a,
        F::Output: Send + 'a,
    {
        // SAFETY: Original implementation missing safety documentation
        unsafe { self.spawn_unchecked(fut) }
    }

    /// Attempts to run a task if at least one is scheduled.
    ///
    /// Running a scheduled task means simply polling its future once.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use edge_executor::Executor;
    ///
    /// let ex: Executor = Default::default();
    /// assert!(!ex.try_tick()); // no tasks to run
    ///
    /// let task = ex.spawn(async {
    ///     println!("Hello world");
    /// });
    /// assert!(ex.try_tick()); // a task was found
    /// ```    
    pub fn try_tick(&self) -> bool {
        if let Some(runnable) = self.try_get_runnable() {
            runnable.run();

            true
        } else {
            false
        }
    }

    /// Runs a single task asynchronously.
    ///
    /// Running a task means simply polling its future once.
    ///
    /// If no tasks are scheduled when this method is called, it will wait until one is scheduled.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use edge_executor::{Executor, block_on};
    ///
    /// let ex: Executor = Default::default();
    ///
    /// let task = ex.spawn(async {
    ///     println!("Hello world");
    /// });
    /// block_on(ex.tick()); // runs the task
    /// ```
    pub async fn tick(&self) {
        poll_fn(|ctx| self.poll_runnable(ctx)).await.run();
    }

    /// Polls the first task scheduled for execution by the executor.
    fn poll_runnable(&self, ctx: &Context<'_>) -> Poll<Runnable> {
        self.state.waker.register(ctx.waker());

        if let Some(runnable) = self.try_get_runnable() {
            Poll::Ready(runnable)
        } else {
            Poll::Pending
        }
    }

    /// Pops the first task scheduled for execution by the executor.
    ///
    /// Returns
    /// - `None` - if no task was scheduled for execution
    /// - `Some(Runnable)` - the first task scheduled for execution. Calling `Runnable::run` will
    ///   execute the task. In other words, it will poll its future.
    #[inline(always)]
    fn try_get_runnable(&self) -> Option<Runnable> {
        self.state.queue.pop()
    }

    /// Runs the executor asynchronously until the given future completes.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use edge_executor::{Executor, block_on};
    ///
    /// let ex: Executor = Default::default();
    ///
    /// let task = ex.spawn(async { 1 + 2 });
    /// let res = block_on(ex.run(async { task.await * 2 }));
    ///
    /// assert_eq!(res, 6);
    /// ```
    pub async fn run<F: Future + Send + 'a>(&self, fut: F) -> F::Output {
        async {
            loop {
                self.tick().await;
            }
        }
        .or(fut)
        .await
    }
}

// SAFETY: Original implementation missing safety documentation
unsafe impl<'a> Send for Executor<'a> {}
// SAFETY: Original implementation missing safety documentation
unsafe impl<'a> Sync for Executor<'a> {}

/// A thread-local executor.
///
/// The executor can only be run on the thread that created it.
///
/// # Examples
///
/// ```ignore
/// use edge_executor::{LocalExecutor, block_on};
///
/// let local_ex: LocalExecutor = Default::default();
///
/// block_on(local_ex.run(async {
///     println!("Hello world!");
/// }));
/// ```
pub struct LocalExecutor<'a> {
    executor: Executor<'a>,
    _not_send: PhantomData<UnsafeCell<&'a Rc<()>>>,
}

impl<'a> Default for LocalExecutor<'a> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> LocalExecutor<'a> {
    /// Creates a single-threaded executor.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use edge_executor::LocalExecutor;
    ///
    /// let local_ex: LocalExecutor = Default::default();
    /// ```
    pub fn new() -> Self {
        Self {
            executor: Executor::new(),
            _not_send: PhantomData,
        }
    }

    /// Spawns a task onto the executor.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use edge_executor::LocalExecutor;
    ///
    /// let local_ex: LocalExecutor = Default::default();
    ///
    /// let task = local_ex.spawn(async {
    ///     println!("Hello world");
    /// });
    /// ```
    ///
    /// Note that if the executor's queue size is equal to the number of currently
    /// spawned and running tasks, spawning this additional task might cause the executor to panic
    /// later, when the task is scheduled for polling.
    pub fn spawn<F>(&self, fut: F) -> Task<F::Output>
    where
        F: Future + 'a,
        F::Output: 'a,
    {
        // SAFETY: Original implementation missing safety documentation
        unsafe { self.executor.spawn_unchecked(fut) }
    }

    /// Attempts to run a task if at least one is scheduled.
    ///
    /// Running a scheduled task means simply polling its future once.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use edge_executor::LocalExecutor;
    ///
    /// let local_ex: LocalExecutor = Default::default();
    /// assert!(!local_ex.try_tick()); // no tasks to run
    ///
    /// let task = local_ex.spawn(async {
    ///     println!("Hello world");
    /// });
    /// assert!(local_ex.try_tick()); // a task was found
    /// ```    
    pub fn try_tick(&self) -> bool {
        self.executor.try_tick()
    }

    /// Runs a single task asynchronously.
    ///
    /// Running a task means simply polling its future once.
    ///
    /// If no tasks are scheduled when this method is called, it will wait until one is scheduled.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use edge_executor::{LocalExecutor, block_on};
    ///
    /// let local_ex: LocalExecutor = Default::default();
    ///
    /// let task = local_ex.spawn(async {
    ///     println!("Hello world");
    /// });
    /// block_on(local_ex.tick()); // runs the task
    /// ```
    pub async fn tick(&self) {
        self.executor.tick().await;
    }

    /// Runs the executor asynchronously until the given future completes.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use vc_tasks::{LocalExecutor, block_on};
    ///
    /// let local_ex: LocalExecutor = Default::default();
    ///
    /// let task = local_ex.spawn(async { 1 + 2 });
    /// let res = block_on(local_ex.run(async { task.await * 2 }));
    ///
    /// assert_eq!(res, 6);
    /// ```
    pub async fn run<F>(&self, fut: F) -> F::Output
    where
        F: Future,
    {
        async {
            loop {
                self.executor.tick().await;
            }
        }
        .or(fut)
        .await
    }
}
