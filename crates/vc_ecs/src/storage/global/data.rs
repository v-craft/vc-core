use alloc::alloc as malloc;
use core::alloc::Layout;
use core::fmt::Debug;
use core::num::NonZeroUsize;
use core::ptr::NonNull;

use vc_ptr::{OwningPtr, Ptr, PtrMut};

use crate::borrow::{UntypedMut, UntypedRef};
use crate::resource::ResourceInfo;
use crate::tick::{Tick, TicksMut, TicksRef};
use crate::utils::DebugName;

// -----------------------------------------------------------------------------
// Drop Guard

/// Drop guard that aborts the process if a resource's drop implementation panics.
struct AbortOnDropFail;

impl Drop for AbortOnDropFail {
    #[cold]
    #[inline(never)]
    fn drop(&mut self) {
        crate::cfg::std! {
            if {
                ::std::eprintln!("Aborting due to drop resource panicked.");
                ::std::process::abort();
            } else {
                panic!("Aborting due to drop resource panicked.");
            }
        }
    }
}

// -----------------------------------------------------------------------------
// ResData

/// Raw storage for a single resource instance.
///
/// Manages memory allocation, initialization state, and change tracking ticks.
pub struct ResData {
    name: DebugName,
    layout: Layout,
    drop_fn: Option<unsafe fn(OwningPtr<'_>)>,
    data: NonNull<u8>,
    added: Tick,
    changed: Tick,
    is_valid: bool,
}

impl Debug for ResData {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ResData")
            .field("name", &self.name)
            .field("is_valid", &self.is_valid)
            .finish()
    }
}

impl Drop for ResData {
    fn drop(&mut self) {
        unsafe {
            self.clear();
        }

        if self.layout.size() != 0 {
            unsafe {
                malloc::dealloc(self.data.as_ptr(), self.layout);
            }
        }
    }
}

impl ResData {
    /// Creates a new `ResData` from resource type information.
    ///
    /// # Safety
    /// - `info` must correctly describe the resource type
    /// - The memory layout must be valid for allocation
    pub(crate) unsafe fn new(info: &ResourceInfo) -> Self {
        let layout = info.layout();
        let drop_fn = info.drop_fn();
        let name = info.debug_name();
        let data = if layout.size() == 0 {
            let align = NonZeroUsize::new(layout.align()).unwrap();
            NonNull::without_provenance(align)
        } else {
            NonNull::new(unsafe { malloc::alloc(layout) })
                .unwrap_or_else(|| malloc::handle_alloc_error(layout))
        };

        Self {
            name,
            layout,
            drop_fn,
            data,
            added: Tick::new(0),
            changed: Tick::new(0),
            is_valid: false,
        }
    }

    /// Returns whether the resource is currently initialized.
    #[inline(always)]
    pub const fn is_valid(&self) -> bool {
        self.is_valid
    }

    /// Returns the debug name of the resource type.
    #[inline(always)]
    pub fn debug_name(&self) -> DebugName {
        self.name
    }

    /// Returns a pointer to the resource data if initialized.
    #[inline]
    pub fn get_data(&self) -> Option<Ptr<'_>> {
        if self.is_valid {
            Some(unsafe { Ptr::new(self.data) })
        } else {
            None
        }
    }

    /// Returns the added tick if the resource is initialized.
    #[inline]
    pub fn get_added(&self) -> Option<Tick> {
        if self.is_valid {
            Some(self.added)
        } else {
            None
        }
    }

    /// Returns the changed tick if the resource is initialized.
    #[inline]
    pub fn get_changed(&self) -> Option<Tick> {
        if self.is_valid {
            Some(self.changed)
        } else {
            None
        }
    }

    /// Removes the resource and returns ownership of its data.
    ///
    /// # Safety
    /// - Caller must ensure the returned `OwningPtr` is properly dropped
    /// - After removal, the resource is marked invalid
    #[inline]
    #[must_use = "The returned ptr should be used."]
    pub unsafe fn remove(&mut self) -> Option<OwningPtr<'_>> {
        if self.is_valid {
            self.is_valid = false;
            unsafe { Some(OwningPtr::new(self.data)) }
        } else {
            None
        }
    }

    /// Drops the resource data if initialized.
    ///
    /// # Safety
    /// - The drop function must be safe to call with the stored data
    /// - Aborts if the drop implementation panics
    /// - If the data is NonSend, the function must be call in correct thread.
    pub unsafe fn clear(&mut self) {
        if self.is_valid
            && let Some(drop_fn) = self.drop_fn
        {
            let guard = AbortOnDropFail;
            self.is_valid = false;
            unsafe {
                drop_fn(OwningPtr::new(self.data));
            }
            ::core::mem::forget(guard);
        }
    }

    /// Inserts a new resource value.
    ///
    /// # Safety
    /// - `value` must point to valid data matching the resource's layout
    /// - `tick` must be a valid system tick
    /// - Previous resource value (if any) is properly dropped
    /// - If the data is NonSend, the function must be call in correct thread.
    pub unsafe fn insert(&mut self, value: OwningPtr<'_>, tick: Tick) {
        if self.is_valid {
            if let Some(drop_fn) = self.drop_fn {
                let guard = AbortOnDropFail;
                unsafe {
                    drop_fn(OwningPtr::new(self.data));
                }
                ::core::mem::forget(guard);
            }
        } else {
            self.is_valid = true;
            self.added = tick;
        }
        self.changed = tick;
        unsafe {
            core::ptr::copy_nonoverlapping::<u8>(
                value.as_ptr(),
                self.data.as_ptr(),
                self.layout.size(),
            );
        }
    }

    /// Returns an untyped reference to the resource if initialized.
    #[inline]
    pub fn get_ref(&self, last_run: Tick, this_run: Tick) -> Option<UntypedRef<'_>> {
        if self.is_valid {
            unsafe { Some(self.untyped_ref(last_run, this_run)) }
        } else {
            None
        }
    }

    /// Returns an untyped mutable reference to the resource if initialized.
    #[inline]
    pub fn get_mut(&mut self, last_run: Tick, this_run: Tick) -> Option<UntypedMut<'_>> {
        if self.is_valid {
            unsafe { Some(self.untyped_mut(last_run, this_run)) }
        } else {
            None
        }
    }

    /// Updates ticks with quick-check logic.
    #[inline(always)]
    pub(super) fn quick_check(&mut self, now: Tick, fall_back: Tick) {
        self.added.quick_check(now, fall_back);
        self.changed.quick_check(now, fall_back);
    }

    /// Returns an untyped reference, panicking if the resource is uninitialized.
    #[inline]
    pub(crate) fn assert_get_ref(&self, last_run: Tick, this_run: Tick) -> UntypedRef<'_> {
        if self.is_valid {
            unsafe { self.untyped_ref(last_run, this_run) }
        } else {
            self.handle_error()
        }
    }

    /// Returns an untyped mutable reference, panicking if the resource is uninitialized.
    #[inline]
    pub(crate) fn assert_get_mut(&mut self, last_run: Tick, this_run: Tick) -> UntypedMut<'_> {
        if self.is_valid {
            unsafe { self.untyped_mut(last_run, this_run) }
        } else {
            self.handle_error()
        }
    }

    #[cold]
    #[inline(never)]
    fn handle_error(&self) -> ! {
        panic!("Resource '{}' was uninitialized.", self.name);
    }

    /// Creates an untyped reference without checking initialization.
    ///
    /// # Safety
    /// - Resource must be initialized (`is_valid == true`)
    /// - Pointers must remain valid for the returned reference's lifetime
    #[inline(always)]
    unsafe fn untyped_ref(&self, last_run: Tick, this_run: Tick) -> UntypedRef<'_> {
        UntypedRef {
            value: unsafe { Ptr::new(self.data) },
            ticks: TicksRef {
                added: &self.added,
                changed: &self.changed,
                last_run,
                this_run,
            },
        }
    }

    /// Creates an untyped mutable reference without checking initialization.
    ///
    /// # Safety
    /// - Resource must be initialized (`is_valid == true`)
    /// - Pointers must remain valid for the returned reference's lifetime
    /// - No other references to the data may exist
    #[inline(always)]
    unsafe fn untyped_mut(&mut self, last_run: Tick, this_run: Tick) -> UntypedMut<'_> {
        UntypedMut {
            value: unsafe { PtrMut::new(self.data) },
            ticks: TicksMut {
                added: &mut self.added,
                changed: &mut self.changed,
                last_run,
                this_run,
            },
        }
    }
}
