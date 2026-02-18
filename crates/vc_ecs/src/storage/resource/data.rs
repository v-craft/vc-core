use core::alloc::Layout;
use core::fmt::Debug;

use vc_ptr::OwningPtr;

use crate::borrow::{UntypedMut, UntypedRef};
use crate::cfg;
use crate::component::{ComponentTicksMut, ComponentTicksRef};
use crate::storage::BlobBox;
use crate::tick::Tick;
use crate::utils::DebugName;

// -----------------------------------------------------------------------------
// ResourceData

pub struct ResourceData {
    name: DebugName,
    data: BlobBox,
    added_tick: Tick,
    changed_tick: Tick,
}

impl Debug for ResourceData {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("ResourceData")
            .field(&self.name)
            .field(&self.data.is_valid())
            .finish()
    }
}

// -----------------------------------------------------------------------------
// NonSendData

#[cfg(feature = "std")]
use std::thread::ThreadId;

pub struct NonSendData {
    name: DebugName,
    data: BlobBox,
    added_tick: Tick,
    changed_tick: Tick,
    #[cfg(feature = "std")]
    thread_id: Option<ThreadId>,
}

impl Debug for NonSendData {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("NonSendData")
            .field(&self.name)
            .field(&self.data.is_valid())
            .finish()
    }
}

impl Drop for NonSendData {
    fn drop(&mut self) {
        if self.data.is_valid() {
            #[cfg(feature = "std")]
            if ::std::thread::panicking() {
                return;
            }
            self.validate_access();
        }
    }
}

// -----------------------------------------------------------------------------
// Basic methods

impl ResourceData {
    /// # Safety
    /// correct drop_fn and layout.
    #[inline]
    pub(super) unsafe fn new(
        name: DebugName,
        layout: Layout,
        drop_fn: Option<unsafe fn(OwningPtr<'_>)>,
    ) -> Self {
        let data = unsafe { BlobBox::new(layout, drop_fn) };
        Self {
            name,
            data,
            added_tick: Tick::new(0),
            changed_tick: Tick::new(0),
        }
    }

    #[inline(always)]
    pub fn debug_name(&self) -> &DebugName {
        &self.name
    }

    /// Return `true` if already contains data.
    #[inline(always)]
    pub fn is_valid(&self) -> bool {
        self.data.is_valid()
    }

    /// Drop data (if exist) and set `is_valid` to `false`.
    #[inline(always)]
    pub fn drop_data(&mut self) {
        self.data.drop_data();
    }

    #[inline]
    pub unsafe fn set_data(&mut self, value: OwningPtr<'_>, tick: Tick) {
        unsafe {
            self.data.set(value);
        }
        self.changed_tick = tick;
        self.added_tick = tick;
    }

    #[inline]
    pub unsafe fn get_ref(&self, last_run: Tick, this_run: Tick) -> UntypedRef<'_> {
        debug_assert!(self.data.is_valid());
        unsafe {
            UntypedRef {
                value: self.data.get(),
                ticks: ComponentTicksRef {
                    added: &self.added_tick,
                    changed: &self.changed_tick,
                    last_run,
                    this_run,
                },
            }
        }
    }

    #[inline]
    pub unsafe fn get_mut(&mut self, last_run: Tick, this_run: Tick) -> UntypedMut<'_> {
        debug_assert!(self.data.is_valid());
        unsafe {
            UntypedMut {
                value: self.data.get_mut(),
                ticks: ComponentTicksMut {
                    added: &mut self.added_tick,
                    changed: &mut self.changed_tick,
                    last_run,
                    this_run,
                },
            }
        }
    }

    #[inline(always)]
    pub(super) fn quick_check(&mut self, now: Tick, fall_back: Tick) {
        self.added_tick.quick_check(now, fall_back);
        self.changed_tick.quick_check(now, fall_back);
    }
}

impl NonSendData {
    #[inline(always)]
    fn validate_access(&self) {
        cfg::std! {
            #[cold]
            #[inline(never)]
            fn invalid_access(this: &NonSendData) -> ! {
                panic!(
                    "Attempted to access or drop non-send resource {} from thread {:?} on a thread {:?}.",
                    this.name,
                    this.thread_id,
                    std::thread::current().id()
                );
            }

            if self.thread_id != Some(std::thread::current().id()) {
                invalid_access(self);
            }
        }
        // Currently, no_std is single-threaded only, so this is safe to ignore.
    }

    #[inline(always)]
    fn init_thread_id(&mut self) {
        cfg::std! {
            self.thread_id = Some(std::thread::current().id());
        }
    }

    /// # Safety
    /// correct drop_fn and layout.
    #[inline]
    pub(super) unsafe fn new(
        name: DebugName,
        layout: Layout,
        drop_fn: Option<unsafe fn(OwningPtr<'_>)>,
    ) -> Self {
        let data = unsafe { BlobBox::new(layout, drop_fn) };

        Self {
            name,
            data,
            added_tick: Tick::new(0),
            changed_tick: Tick::new(0),
            #[cfg(feature = "std")]
            thread_id: None,
        }
    }

    /// Drop data (if exist) and set `is_valid` to `false`.
    #[inline(always)]
    pub fn drop_data(&mut self) {
        self.validate_access();
        self.data.drop_data();
    }

    #[inline(always)]
    pub fn debug_name(&self) -> &DebugName {
        &self.name
    }

    /// Return `true` if already contains data.
    #[inline(always)]
    pub fn is_valid(&self) -> bool {
        self.data.is_valid()
    }

    #[inline]
    pub unsafe fn set_data(&mut self, value: OwningPtr<'_>, tick: Tick) {
        if self.is_valid() {
            self.validate_access();
        }
        self.init_thread_id();
        unsafe {
            self.data.set(value);
        }
        self.changed_tick = tick;
        self.added_tick = tick;
    }

    #[inline]
    pub unsafe fn get_ref(&self, last_run: Tick, this_run: Tick) -> UntypedRef<'_> {
        debug_assert!(self.data.is_valid());
        self.validate_access();
        unsafe {
            UntypedRef {
                value: self.data.get(),
                ticks: ComponentTicksRef {
                    added: &self.added_tick,
                    changed: &self.changed_tick,
                    last_run,
                    this_run,
                },
            }
        }
    }

    #[inline]
    pub unsafe fn get_mut(&mut self, last_run: Tick, this_run: Tick) -> UntypedMut<'_> {
        debug_assert!(self.data.is_valid());
        self.validate_access();
        unsafe {
            UntypedMut {
                value: self.data.get_mut(),
                ticks: ComponentTicksMut {
                    added: &mut self.added_tick,
                    changed: &mut self.changed_tick,
                    last_run,
                    this_run,
                },
            }
        }
    }

    #[inline(always)]
    pub(super) fn quick_check(&mut self, now: Tick, fall_back: Tick) {
        self.added_tick.quick_check(now, fall_back);
        self.changed_tick.quick_check(now, fall_back);
    }
}
