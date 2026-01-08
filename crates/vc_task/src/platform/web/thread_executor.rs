use async_task::Task;

// -----------------------------------------------------------------------------
// ThreadExecutor

use core::marker::PhantomData;

/// An executor that can only be ticked on the thread it was
/// instantiated on. But can spawn `Send` tasks from other threads.
/// 
/// This is a dummy struct for no_std support to provide
/// the same api as with the multithreaded task pool.
/// 
/// # Panics
/// Panic if call it's spawn or tick function.
#[derive(Debug)]
pub struct ThreadExecutor<'a>(PhantomData<&'a ()>);

impl<'task> Default for ThreadExecutor<'task> {
    #[inline(always)]
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<'task> ThreadExecutor<'task> {
    /// Creates a new `ThreadExecutor`
    #[inline(always)]
    pub fn new() -> Self {
        Self(PhantomData)
    }

    /// Spawn a task on the thread executor
    /// 
    /// # Panics
    /// Panic if this function be used in wasm env.
    #[deprecated = "`ThreadExecutor` cannot be used in wasm env."]
    pub fn spawn<T: Send + 'task>(
        &self,
        _future: impl Future<Output = T> + Send + 'task,
    ) -> Task<T> {
        unreachable!("`ThreadExecutor` cannot be used in wasm env.")
    }

    /// Gets the [`ThreadExecutorTicker`] for this executor.
    /// 
    /// # Panics
    /// Panic if this function be used in wasm env.
    #[deprecated = "`ThreadExecutor` cannot be used in wasm env."]
    pub fn ticker<'ticker>(&'ticker self) -> Option<ThreadExecutorTicker<'task, 'ticker>> {
        unreachable!("`ThreadExecutor` cannot be used in wasm env.")
    }

    /// Returns true if `self` and `other`'s executor is same.
    #[inline(always)]
    pub fn is_same(&self, other: &Self) -> bool {
        core::ptr::eq(self, other)
    }
}

// -----------------------------------------------------------------------------
// ThreadExecutorTicker

/// Used to tick the [`ThreadExecutor`].
/// 
/// The executor does not make progress unless it is
/// manually ticked on the thread it was created on.
/// 
/// Cannot be used in wasm env.
#[derive(Debug)]
pub struct ThreadExecutorTicker<'task, 'ticker> {
    _executor: &'ticker ThreadExecutor<'task>,
    // make type not send or sync
    _marker: PhantomData<*const ()>,
}

impl<'task, 'ticker> ThreadExecutorTicker<'task, 'ticker> {
    /// Tick the thread executor.
    /// 
    /// # Panics
    /// Panic if this function be used in wasm env.
    #[deprecated = "`ThreadExecutor` cannot be used in wasm env."]
    pub async fn tick(&self) {
        unreachable!("`ThreadExecutorTicker` cannot be used in wasm env.")
    }

    /// Synchronously try to tick a task on the executor.
    /// 
    /// Returns false if does not find a task to tick.
    /// 
    /// # Panics
    /// Panic if this function be used in wasm env.
    #[deprecated = "`ThreadExecutor` cannot be used in wasm env."]
    pub fn try_tick(&self) -> bool {
        unreachable!("`ThreadExecutorTicker` cannot be used in wasm env.")
    }
}


