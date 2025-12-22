#![cfg_attr(docsrs, feature(doc_cfg))]
#![no_std]

// -----------------------------------------------------------------------------
// Compilation config

pub mod cfg {
    pub use vc_os::cfg::{std, web};
    vc_cfg::define_alias! {
        #[cfg(target_arch = "wasm32")] => {
            /// Indicates the current target requires additional `?Send` bounds.
            optional_send
        }
    }
}

// -----------------------------------------------------------------------------
// no_std support

extern crate alloc;

// -----------------------------------------------------------------------------
// Modules

pub mod mini_executor;

// -----------------------------------------------------------------------------
// Top-Level Exports

/// Blocks on the supplied `future`.
/// This implementation will busy-wait until it is completed.
/// Consider enabling the `async-io` or `futures-lite` features.
pub fn block_on<T>(future: impl Future<Output = T>) -> T {
    use core::task::{Context, Poll};

    // Pin the future on the stack.
    let mut future = core::pin::pin!(future);

    // We don't care about the waker as we're just going to poll as fast as possible.
    let cx = &mut Context::from_waker(core::task::Waker::noop());

    // Keep polling until the future is ready.
    loop {
        match future.as_mut().poll(cx) {
            Poll::Ready(output) => return output,
            Poll::Pending => core::hint::spin_loop(),
        }
    }
}
