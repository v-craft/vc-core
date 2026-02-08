#![expect(unsafe_code, reason = "original implementation")]

use core::alloc::Layout;
use core::cell::UnsafeCell;
use core::panic::Location;

use vc_ptr::{OwningPtr, Ptr};
use vc_utils::UnsafeCellDeref;

use crate::cfg;
use crate::component::{ComponentTickCells, ComponentTicks, ComponentTicksMut, MutUntyped};
use crate::storage::BlobArray;
use crate::tick::{CheckTicks, Tick};
use crate::utils::{DebugLocation, DebugName};

// -----------------------------------------------------------------------------
// ResourceData

pub struct ResourceData {
    name: DebugName,
    /// Capacity is 1, length is 1 if `present` and 0 otherwise.
    data: BlobArray,
    is_present: bool,
    added_tick: UnsafeCell<Tick>,
    changed_tick: UnsafeCell<Tick>,
    changed_by: DebugLocation<UnsafeCell<&'static Location<'static>>>,
}

impl Drop for ResourceData {
    fn drop(&mut self) {
        unsafe {
            self.data.dealloc(1, self.is_present as usize);
        }
    }
}

impl ResourceData {
    const INDEX: usize = 0;

    #[inline(always)]
    pub fn debug_name(&self) -> &DebugName {
        &self.name
    }

    #[inline]
    pub fn new(name: DebugName, layout: Layout, drop_fn: Option<unsafe fn(OwningPtr<'_>)>) -> Self {
        let data = unsafe { BlobArray::with_capacity(layout, drop_fn, 1) };

        Self {
            name,
            data,
            is_present: false,
            added_tick: UnsafeCell::new(Tick::new(0)),
            changed_tick: UnsafeCell::new(Tick::new(0)),
            changed_by: DebugLocation::caller().map(UnsafeCell::new),
        }
    }

    #[inline(always)]
    pub fn is_present(&self) -> bool {
        self.is_present
    }

    #[inline]
    pub fn get_data(&self) -> Option<Ptr<'_>> {
        if self.is_present {
            unsafe { Some(self.data.get_first()) }
        } else {
            None
        }
    }

    #[inline]
    pub fn get_component_ticks(&self) -> Option<ComponentTicks> {
        if self.is_present {
            Some(ComponentTicks {
                added: unsafe { self.added_tick.read() },
                changed: unsafe { self.changed_tick.read() },
            })
        } else {
            None
        }
    }

    #[inline]
    pub fn get_data_with_ticks(&self) -> Option<(Ptr<'_>, ComponentTickCells<'_>)> {
        if self.is_present {
            Some((
                unsafe { self.data.get_first() },
                ComponentTickCells {
                    added: &self.added_tick,
                    changed: &self.changed_tick,
                    changed_by: self.changed_by.as_ref(),
                },
            ))
        } else {
            None
        }
    }

    #[inline]
    pub fn get_mut(&mut self, last_run: Tick, this_run: Tick) -> Option<MutUntyped<'_>> {
        if self.is_present {
            let value = unsafe { self.data.get_first_mut() };
            let cells = ComponentTickCells {
                added: &self.added_tick,
                changed: &self.changed_tick,
                changed_by: self.changed_by.as_ref(),
            };
            let ticks = unsafe { ComponentTicksMut::from_tick_cells(cells, last_run, this_run) };
            Some(MutUntyped { value, ticks })
        } else {
            None
        }
    }

    #[inline]
    pub unsafe fn insert(
        &mut self,
        value: OwningPtr<'_>,
        change_tick: Tick,
        _caller: DebugLocation,
    ) {
        if self.is_present {
            unsafe {
                self.data.replace_item(Self::INDEX, value);
            }
        } else {
            unsafe {
                self.data.init_item(Self::INDEX, value);
            }
            self.is_present = true;
        }

        unsafe {
            *self.changed_tick.deref_mut() = change_tick;
        }

        cfg::debug! {
            self.changed_by.as_ref()
                .map(|changed_by| unsafe{ changed_by.deref_mut() })
                .assign(_caller);
        }
    }

    #[inline]
    pub unsafe fn insert_with_ticks(
        &mut self,
        value: OwningPtr<'_>,
        change_ticks: ComponentTicks,
        _caller: DebugLocation,
    ) {
        if self.is_present {
            unsafe {
                self.data.replace_item(Self::INDEX, value);
            }
        } else {
            unsafe {
                self.data.init_item(Self::INDEX, value);
            }
            self.is_present = true;
        }

        unsafe {
            *self.added_tick.deref_mut() = change_ticks.added;
            *self.changed_tick.deref_mut() = change_ticks.changed;
        }

        cfg::debug! {
            self.changed_by.as_ref()
                .map(|changed_by| unsafe{ changed_by.deref_mut() })
                .assign(_caller);
        }
    }

    #[inline]
    #[must_use = "The returned pointer to the removed component should be used or dropped"]
    pub fn remove(&mut self) -> Option<(OwningPtr<'_>, ComponentTicks, DebugLocation)> {
        if !self.is_present {
            return None;
        }

        self.is_present = false;

        unsafe {
            let ptr = self.data.remove_last(Self::INDEX);
            let ticks = ComponentTicks {
                added: self.added_tick.read(),
                changed: self.changed_tick.read(),
            };
            let caller = self.changed_by.as_ref().map(|changed_by| changed_by.read());

            Some((ptr, ticks, caller))
        }
    }

    #[inline]
    pub fn remove_and_drop(&mut self) {
        if self.is_present {
            unsafe {
                self.data.drop_last(Self::INDEX);
            }
            self.is_present = false;
        }
    }

    #[inline]
    pub fn check_ticks(&mut self, check: CheckTicks) {
        self.added_tick.get_mut().check_age(check.tick());
        self.changed_tick.get_mut().check_age(check.tick());
    }
}

// -----------------------------------------------------------------------------
// NoSendResourceData

#[cfg(feature = "std")]
use std::thread::ThreadId;

pub struct NoSendResourceData {
    name: DebugName,
    /// Capacity is 1, length is 1 if `present` and 0 otherwise.
    data: BlobArray,
    is_present: bool,
    added_tick: UnsafeCell<Tick>,
    changed_tick: UnsafeCell<Tick>,
    changed_by: DebugLocation<UnsafeCell<&'static Location<'static>>>,
    #[cfg(feature = "std")]
    thread_id: Option<ThreadId>,
}

impl Drop for NoSendResourceData {
    fn drop(&mut self) {
        unsafe {
            self.data.dealloc(1, self.is_present as usize);
        }
    }
}

impl NoSendResourceData {
    const INDEX: usize = 0;

    #[inline(always)]
    fn validate_access(&self) {
        cfg::std! {
            #[cold]
            #[inline(never)]
            fn invalid_access(this: &NoSendResourceData) -> ! {
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

    #[inline(always)]
    pub fn debug_name(&self) -> &DebugName {
        &self.name
    }

    #[inline]
    pub fn new(name: DebugName, layout: Layout, drop_fn: Option<unsafe fn(OwningPtr<'_>)>) -> Self {
        let data = unsafe { BlobArray::with_capacity(layout, drop_fn, 1) };

        Self {
            name,
            data,
            is_present: false,
            added_tick: UnsafeCell::new(Tick::new(0)),
            changed_tick: UnsafeCell::new(Tick::new(0)),
            changed_by: DebugLocation::caller().map(UnsafeCell::new),
            #[cfg(feature = "std")]
            thread_id: None,
        }
    }

    #[inline(always)]
    pub fn is_present(&self) -> bool {
        self.is_present
    }

    #[inline]
    pub fn get_data(&self) -> Option<Ptr<'_>> {
        if self.is_present {
            self.validate_access();
            unsafe { Some(self.data.get_first()) }
        } else {
            None
        }
    }

    #[inline]
    pub fn get_component_ticks(&self) -> Option<ComponentTicks> {
        if self.is_present {
            self.validate_access();
            Some(ComponentTicks {
                added: unsafe { self.added_tick.read() },
                changed: unsafe { self.changed_tick.read() },
            })
        } else {
            None
        }
    }

    #[inline]
    pub fn get_data_with_ticks(&self) -> Option<(Ptr<'_>, ComponentTickCells<'_>)> {
        if self.is_present {
            self.validate_access();
            Some((
                unsafe { self.data.get_first() },
                ComponentTickCells {
                    added: &self.added_tick,
                    changed: &self.changed_tick,
                    changed_by: self.changed_by.as_ref(),
                },
            ))
        } else {
            None
        }
    }

    #[inline]
    pub fn get_mut(&mut self, last_run: Tick, this_run: Tick) -> Option<MutUntyped<'_>> {
        if self.is_present {
            self.validate_access();
            let value = unsafe { self.data.get_first_mut() };
            let cells = ComponentTickCells {
                added: &self.added_tick,
                changed: &self.changed_tick,
                changed_by: self.changed_by.as_ref(),
            };
            let ticks = unsafe { ComponentTicksMut::from_tick_cells(cells, last_run, this_run) };
            Some(MutUntyped { value, ticks })
        } else {
            None
        }
    }

    #[inline]
    pub unsafe fn insert(
        &mut self,
        value: OwningPtr<'_>,
        change_tick: Tick,
        _caller: DebugLocation,
    ) {
        if self.is_present {
            self.validate_access();
            unsafe {
                self.data.replace_item(Self::INDEX, value);
            }
        } else {
            self.init_thread_id();
            unsafe {
                self.data.init_item(Self::INDEX, value);
            }
            self.is_present = true;
        }

        unsafe {
            *self.changed_tick.deref_mut() = change_tick;
        }

        cfg::debug! {
            self.changed_by.as_ref()
                .map(|changed_by| unsafe{ changed_by.deref_mut() })
                .assign(_caller);
        }
    }

    #[inline]
    pub unsafe fn insert_with_ticks(
        &mut self,
        value: OwningPtr<'_>,
        change_ticks: ComponentTicks,
        _caller: DebugLocation,
    ) {
        if self.is_present {
            self.validate_access();
            unsafe {
                self.data.replace_item(Self::INDEX, value);
            }
        } else {
            self.init_thread_id();
            unsafe {
                self.data.init_item(Self::INDEX, value);
            }
            self.is_present = true;
        }

        unsafe {
            *self.added_tick.deref_mut() = change_ticks.added;
            *self.changed_tick.deref_mut() = change_ticks.changed;
        }

        cfg::debug! {
            self.changed_by.as_ref()
                .map(|changed_by| unsafe{ changed_by.deref_mut() })
                .assign(_caller);
        }
    }

    #[inline]
    #[must_use = "The returned pointer to the removed component should be used or dropped"]
    pub fn remove(&mut self) -> Option<(OwningPtr<'_>, ComponentTicks, DebugLocation)> {
        if !self.is_present {
            return None;
        }

        self.validate_access();
        self.is_present = false;

        unsafe {
            let ptr = self.data.remove_last(Self::INDEX);
            let ticks = ComponentTicks {
                added: self.added_tick.read(),
                changed: self.changed_tick.read(),
            };
            let caller = self.changed_by.as_ref().map(|changed_by| changed_by.read());

            Some((ptr, ticks, caller))
        }
    }

    #[inline]
    pub fn remove_and_drop(&mut self) {
        if self.is_present {
            self.validate_access();
            unsafe {
                self.data.drop_last(Self::INDEX);
            }
            self.is_present = false;
        }
    }

    #[inline]
    pub fn check_ticks(&mut self, check: CheckTicks) {
        self.added_tick.get_mut().check_age(check.tick());
        self.changed_tick.get_mut().check_age(check.tick());
    }
}
