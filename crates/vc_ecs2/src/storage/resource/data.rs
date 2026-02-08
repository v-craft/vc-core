#![allow(unused_variables, reason = "`DebugLocation` is unused in release mod.")]

use core::alloc::Layout;
use core::cell::UnsafeCell;
use core::fmt::Debug;
use core::panic::Location;

use vc_ptr::OwningPtr;

use crate::cfg;
use crate::storage::BlobBox;
use crate::tick::{CheckTicks, Tick};
use crate::utils::{DebugLocation, DebugName};

// -----------------------------------------------------------------------------
// ResourceData

pub struct ResourceData {
    name: DebugName,
    data: BlobBox,
    added_tick: UnsafeCell<Tick>,
    changed_tick: UnsafeCell<Tick>,
    changed_by: DebugLocation<UnsafeCell<&'static Location<'static>>>,
}

impl Debug for ResourceData {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ResData")
            .field("name", &self.name)
            .field("is_valid", &self.is_valid())
            .finish()
    }
}

// -----------------------------------------------------------------------------
// NoSendData

#[cfg(feature = "std")]
use std::thread::ThreadId;

pub struct NoSendData {
    name: DebugName,
    data: BlobBox,
    added_tick: UnsafeCell<Tick>,
    changed_tick: UnsafeCell<Tick>,
    changed_by: DebugLocation<UnsafeCell<&'static Location<'static>>>,
    #[cfg(feature = "std")]
    thread_id: Option<ThreadId>,
}

impl Debug for NoSendData {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("NoSendData")
            .field("name", &self.name)
            .field("is_valid", &self.is_valid())
            .finish()
    }
}

impl Drop for NoSendData {
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
            added_tick: UnsafeCell::new(Tick::new(0)),
            changed_tick: UnsafeCell::new(Tick::new(0)),
            changed_by: DebugLocation::caller().map(UnsafeCell::new),
        }
    }

    /// Drop data (if exist) and set `is_valid` to `false`.
    #[inline(always)]
    pub(super) fn drop_data(&mut self) {
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

    /// # Safety:
    /// - `self.is_valid() == false`.
    /// - `OwingPtr` points to valid data.
    #[inline]
    pub unsafe fn init(&mut self, value: OwningPtr<'_>, tick: Tick, caller: DebugLocation) {
        assert!(!self.data.is_valid());

        unsafe {
            self.data.set(value);
        }
        *self.changed_tick.get_mut() = tick;
        *self.added_tick.get_mut() = tick;

        cfg::debug! {
            self.changed_by.as_mut()
                .map(|cb| cb.get_mut())
                .assign(caller);
        }
    }

    #[inline]
    pub fn check_ticks(&mut self, check: CheckTicks) {
        let now = check.tick();
        let fall_back = now.relative_to(Tick::MAX_AGE);
        self.quick_check(now, fall_back);
    }

    #[inline(always)]
    pub(super) fn quick_check(&mut self, now: Tick, fall_back: Tick) {
        self.added_tick.get_mut().quick_check(now, fall_back);
        self.changed_tick.get_mut().quick_check(now, fall_back);
    }
}

impl NoSendData {
    #[inline(always)]
    fn validate_access(&self) {
        cfg::std! {
            #[cold]
            #[inline(never)]
            fn invalid_access(this: &NoSendData) -> ! {
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
            added_tick: UnsafeCell::new(Tick::new(0)),
            changed_tick: UnsafeCell::new(Tick::new(0)),
            changed_by: DebugLocation::caller().map(UnsafeCell::new),
            #[cfg(feature = "std")]
            thread_id: None,
        }
    }

    /// Drop data (if exist) and set `is_valid` to `false`.
    #[inline(always)]
    pub(super) fn drop_data(&mut self) {
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

    /// # Safety:
    /// - `self.is_valid() == false`.
    /// - `OwingPtr` points to valid data.
    #[inline]
    pub unsafe fn init(&mut self, value: OwningPtr<'_>, tick: Tick, caller: DebugLocation) {
        assert!(!self.data.is_valid());
        self.init_thread_id();

        unsafe {
            self.data.set(value);
        }
        *self.changed_tick.get_mut() = tick;
        *self.added_tick.get_mut() = tick;
        cfg::debug! {
            self.changed_by.as_mut()
                .map(|cb| cb.get_mut())
                .assign(caller);
        }
    }

    #[inline]
    pub fn check_ticks(&mut self, check: CheckTicks) {
        let now = check.tick();
        let fall_back = now.relative_to(Tick::MAX_AGE);
        self.quick_check(now, fall_back);
    }

    #[inline(always)]
    pub(super) fn quick_check(&mut self, now: Tick, fall_back: Tick) {
        self.added_tick.get_mut().quick_check(now, fall_back);
        self.changed_tick.get_mut().quick_check(now, fall_back);
    }
}

// -----------------------------------------------------------------------------
// Optional methods

impl ResourceData {
    // /// # Safety:
    // /// - `self.is_valid() == true`.
    // /// - `OwingPtr` points to valid data.
    // pub unsafe fn updata(
    //     &mut self,
    //     value: OwningPtr<'_>,
    //     tick: Tick,
    //     caller: DebugLocation,
    // ) {
    //     assert!(self.data.is_valid());

    //     unsafe { self.data.set(value); }
    //     *self.changed_tick.get_mut() = tick;

    //     cfg::debug! {
    //         self.changed_by.as_mut()
    //             .map(|cb| cb.get_mut())
    //             .assign(caller);
    //     }
    // }

    // /// # Safety:
    // /// - `self.is_valid() == true`.
    // #[inline(always)]
    // pub unsafe fn get_ptr(&self) -> Ptr<'_> {
    //     unsafe { self.data.get() }
    // }

    // /// # Safety:
    // /// - `self.is_valid() == true`.
    // #[inline(always)]
    // pub unsafe fn get_ptr_mut(&mut self) -> PtrMut<'_> {
    //     unsafe { self.data.get_mut() }
    // }

    // /// # Safety:
    // /// - `self.is_valid() == true`.
    // #[must_use = "The returned pointer should be used to move the data"]
    // #[inline(always)]
    // pub unsafe fn take(&mut self) -> OwningPtr<'_> {
    //     unsafe { self.data.take() }
    // }

    // #[inline(always)]
    // pub fn get_ticks(&self) -> ComponentTicks {
    //     ComponentTicks {
    //         added: unsafe{ *self.added_tick.get() },
    //         changed: unsafe { *self.changed_tick.get() },
    //     }
    // }

    // #[inline(always)]
    // pub fn get_changed_by(&self) -> DebugLocation {
    //     self.changed_by.get_inner()
    // }

    // /// # Safety:
    // /// - `self.is_valid() == true`.
    // #[inline(always)]
    // pub unsafe fn get_untyped(&self, last_run: Tick, this_run: Tick) -> UntypedRef<'_> {
    //     unsafe {
    //         UntypedRef {
    //             value: self.data.get(),
    //             ticks: ComponentTicksRef {
    //                 added: &*self.added_tick.get(),
    //                 changed: &*self.changed_tick.get(),
    //                 changed_by: self.changed_by.get_ref(),
    //                 last_run,
    //                 this_run,
    //             },
    //         }
    //     }
    // }

    // /// # Safety:
    // /// - `self.is_valid() == true`.
    // #[inline(always)]
    // pub unsafe fn get_untyped_mut(&mut self, last_run: Tick, this_run: Tick) -> UntypedMut<'_> {
    //     unsafe {
    //         UntypedMut {
    //             value: self.data.get_mut(),
    //             ticks: ComponentTicksMut {
    //                 added: self.added_tick.get_mut(),
    //                 changed: self.changed_tick.get_mut(),
    //                 changed_by: self.changed_by.get_mut(),
    //                 last_run,
    //                 this_run,
    //             },
    //         }
    //     }
    // }
}

impl NoSendData {
    // /// # Safety:
    // /// - `self.is_valid() == true`.
    // /// - `OwingPtr` points to valid data.
    // /// - Run on the correct thread.
    // pub unsafe fn updata(
    //     &mut self,
    //     value: OwningPtr<'_>,
    //     tick: Tick,
    //     caller: DebugLocation,
    // ) {
    //     assert!(self.data.is_valid());
    //     self.validate_access();

    //     unsafe { self.data.set(value); }
    //     *self.changed_tick.get_mut() = tick;
    //     cfg::debug! {
    //         self.changed_by.as_mut()
    //             .map(|cb| cb.get_mut())
    //             .assign(caller);
    //     }
    // }

    // /// # Safety:
    // /// - `self.is_valid() == true`.
    // /// - Run on the correct thread.
    // #[inline(always)]
    // pub unsafe fn get_ptr(&self) -> Ptr<'_> {
    //     self.validate_access();
    //     unsafe { self.data.get() }
    // }

    // /// # Safety:
    // /// - `self.is_valid() == true`.
    // /// - Run on the correct thread.
    // #[inline(always)]
    // pub unsafe fn get_ptr_mut(&mut self) -> PtrMut<'_> {
    //     self.validate_access();
    //     unsafe { self.data.get_mut() }
    // }

    // /// # Safety:
    // /// - `self.is_valid() == true`.
    // /// - Run on the correct thread.
    // #[must_use = "The returned pointer should be used to move the data"]
    // #[inline(always)]
    // pub unsafe fn take(&mut self) -> OwningPtr<'_> {
    //     self.validate_access();
    //     unsafe { self.data.take() }
    // }

    // #[inline(always)]
    // pub fn get_ticks(&self) -> ComponentTicks {
    //     ComponentTicks {
    //         added: unsafe{ *self.added_tick.get() },
    //         changed: unsafe { *self.changed_tick.get() },
    //     }
    // }

    // #[inline(always)]
    // pub fn get_changed_by(&self) -> DebugLocation {
    //     self.changed_by.get_inner()
    // }

    // /// # Safety:
    // /// - `self.is_valid() == true`.
    // /// - Run on the correct thread.
    // #[inline(always)]
    // pub unsafe fn get_untyped(&self, last_run: Tick, this_run: Tick) -> UntypedRef<'_> {
    //     self.validate_access();
    //     unsafe {
    //         UntypedRef {
    //             value: self.data.get(),
    //             ticks: ComponentTicksRef {
    //                 added: &*self.added_tick.get(),
    //                 changed: &*self.changed_tick.get(),
    //                 changed_by: self.changed_by.get_ref(),
    //                 last_run,
    //                 this_run,
    //             },
    //         }
    //     }
    // }

    // /// # Safety:
    // /// - `self.is_valid() == true`.
    // /// - Run on the correct thread.
    // #[inline(always)]
    // pub unsafe fn get_untyped_mut(&mut self, last_run: Tick, this_run: Tick) -> UntypedMut<'_> {
    //     self.validate_access();
    //     unsafe {
    //         UntypedMut {
    //             value: self.data.get_mut(),
    //             ticks: ComponentTicksMut {
    //                 added: self.added_tick.get_mut(),
    //                 changed: self.changed_tick.get_mut(),
    //                 changed_by: self.changed_by.get_mut(),
    //                 last_run,
    //                 this_run,
    //             },
    //         }
    //     }
    // }
}
