/// A guard used to terminate a process
/// when memory allocation failure.
pub(crate) struct AbortOnPanic;

impl Drop for AbortOnPanic {
    #[cold]
    #[inline(never)]
    fn drop(&mut self) {
        crate::cfg::std! {
            if {
                std::eprintln!("Aborting due to allocator error.");
                std::process::abort();
            } else {
                panic!("Aborting due to allocator error.");
            }
        }
    }
}
