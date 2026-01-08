use core::marker::PhantomData;

use async_task::Task;

// -----------------------------------------------------------------------------
// Scope Executor

/// This is a dummy struct for wasm support to provide
/// the same api as with the multithreaded task pool.
///
/// # Panics
/// Panic if call it's spawn or tick function.
#[derive(Debug)]
pub struct ScopeExecutor<'a> {
    _marker: PhantomData<&'a ()>,
}

impl<'task> Default for ScopeExecutor<'task> {
    #[inline(always)]
    fn default() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<'task> ScopeExecutor<'task> {
    /// Creates a new `ScopeExecutor`
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }

    /// Spawn a task on the thread executor
    ///
    /// # Panics
    ///
    /// Panic if this function be used in `wasm` env.
    #[deprecated = "`ScopeExecutor` cannot be used in wasm env."]
    pub fn spawn<T: Send + 'task>(
        &self,
        _future: impl Future<Output = T> + Send + 'task,
    ) -> Task<T> {
        unreachable!("`ScopeExecutor` cannot be used in wasm env.")
    }

    /// Gets the [`ScopeExecutorTicker`] for this executor.
    ///
    /// # Panics
    ///
    /// Panic if this function be used in `wasm` env.
    #[deprecated = "`ScopeExecutor` cannot be used in wasm env."]
    pub fn ticker<'ticker>(&'ticker self) -> Option<ScopeExecutorTicker<'task, 'ticker>> {
        unreachable!("`ScopeExecutor` cannot be used in wasm env.")
    }

    /// Returns true if `self` and `other`'s executor is same.
    #[inline(always)]
    pub fn is_same(&self, other: &Self) -> bool {
        core::ptr::eq(self, other)
    }
}

// -----------------------------------------------------------------------------
// ScopeExecutorTicker

/// Used to tick the [`ScopeExecutor`].
///
/// The executor does not make progress unless it is
/// manually ticked on the thread it was created on.
///
/// Cannot be used in `wasm` env.
#[derive(Debug)]
pub struct ScopeExecutorTicker<'task, 'ticker> {
    _executor: &'ticker ScopeExecutor<'task>,
    // make type not send or sync
    _marker: PhantomData<*const ()>,
}

impl<'task, 'ticker> ScopeExecutorTicker<'task, 'ticker> {
    /// Tick the thread executor.
    ///
    /// # Panics
    ///
    /// Panic if this function be used in `wasm` env.
    #[deprecated = "`ScopeExecutor` cannot be used in wasm env."]
    pub async fn tick(&self) {
        unreachable!("`ScopeExecutor` cannot be used in wasm env.")
    }

    /// Synchronously try to tick a task on the executor.
    ///
    /// Returns false if does not find a task to tick.
    ///
    /// # Panics
    ///
    /// Panic if this function be used in `wasm` env.
    #[deprecated = "`ScopeExecutor` cannot be used in wasm env."]
    pub fn try_tick(&self) -> bool {
        unreachable!("`ScopeExecutor` cannot be used in wasm env.")
    }
}
