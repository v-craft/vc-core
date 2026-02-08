use alloc::borrow::ToOwned;

use crate::tick::Tick;
use crate::utils::DebugLocation;

// -----------------------------------------------------------------------------
// DetectChanges

pub trait DetectChanges {
    /// Returns `true` if this value was added after the system last ran.
    fn is_added(&self) -> bool;

    /// Returns `true` if this value was added or mutably dereferenced
    /// either since the last time the system ran or, if the system never ran,
    /// since the beginning of the program.
    ///
    /// To check if the value was mutably dereferenced only,
    /// use `this.is_changed() && !this.is_added()`.
    fn is_changed(&self) -> bool;

    /// Returns the change tick recording the time this data was added.
    fn added_tick(&self) -> Tick;

    /// Returns the change tick recording the time this data was most recently changed.
    ///
    /// Note that components and resources are also marked as changed upon insertion.
    fn changed_tick(&self) -> Tick;

    /// The location that last caused this to change.
    fn changed_by(&self) -> DebugLocation;
}

// -----------------------------------------------------------------------------
// DetectChangesMut

pub trait DetectChangesMut: DetectChanges {
    /// The type contained within this smart pointer
    ///
    /// For example, for `ResMut<T>` this would be `T`.
    type Inner: ?Sized;

    /// Flags this value as having been changed.
    ///
    /// Mutably accessing this smart pointer will automatically flag this value as having been changed.
    /// However, mutation through interior mutability requires manual reporting.
    ///
    /// **Note**: This operation cannot be undone.
    fn set_changed(&mut self);

    /// Flags this value as having been added.
    ///
    /// It is not normally necessary to call this method.
    /// The 'added' tick is set when the value is first added,
    /// and is not normally changed afterwards.
    ///
    /// **Note**: This operation cannot be undone.
    fn set_added(&mut self);

    /// Manually sets the change tick recording the time when this data was last mutated.
    ///
    /// # Warning
    /// This is a complex and error-prone operation, primarily intended for use with
    /// rollback networking strategies. If you merely want to flag this data as changed,
    /// use [`set_changed`] instead. If you want to avoid triggering change detection,
    /// use [`bypass_change_detection`] instead.
    ///
    /// [`set_changed`]: DetectChangesMut::set_changed
    /// [`bypass_change_detection`]: DetectChangesMut::bypass_change_detection
    fn set_changed_with(&mut self, changed: Tick);

    /// Manually sets the added tick recording the time when this data was last added.
    ///
    /// # Warning
    /// The caveats of [`set_changed`](DetectChangesMut::set_changed) apply.
    /// This modifies both the added and changed ticks together.
    fn set_added_with(&mut self, added: Tick);

    /// Manually bypasses change detection, allowing you to mutate the underlying value
    /// without updating the change tick.
    ///
    /// # Warning
    /// This is a risky operation, that can have unexpected consequences on any system relying on this code.
    /// However, it can be an essential escape hatch when, for example,
    /// you are trying to synchronize representations using change detection and need to avoid infinite recursion.
    fn bypass_change_detection(&mut self) -> &mut Self::Inner;

    /// Overwrites this smart pointer with the given value, if and only if `*self != value`.
    /// Returns `true` if the value was overwritten, and returns `false` if it was not.
    #[inline]
    #[track_caller]
    fn set_if_neq(&mut self, value: Self::Inner) -> bool
    where
        Self::Inner: Sized + PartialEq,
    {
        let old = self.bypass_change_detection();
        if *old != value {
            *old = value;
            self.set_changed();
            true
        } else {
            false
        }
    }

    /// Overwrites this smart pointer with the given value, if and only if `*self != value`,
    /// returning the previous value if this occurs.
    #[inline]
    #[must_use = "If you don't need to handle the previous value, use `set_if_neq` instead."]
    fn replace_if_neq(&mut self, value: Self::Inner) -> Option<Self::Inner>
    where
        Self::Inner: Sized + PartialEq,
    {
        let old = self.bypass_change_detection();
        if *old != value {
            let previous = core::mem::replace(old, value);
            self.set_changed();
            Some(previous)
        } else {
            None
        }
    }

    /// Overwrites this smart pointer with a clone of the given value, if and only if `*self != value`.
    /// Returns `true` if the value was overwritten, and returns `false` if it was not.
    ///
    /// This method is useful when the caller only has a borrowed form of `Inner`,
    /// e.g. when writing a `&str` into a `Mut<String>`.
    fn clone_from_if_neq<T>(&mut self, value: &T) -> bool
    where
        T: ToOwned<Owned = Self::Inner> + ?Sized,
        Self::Inner: PartialEq<T>,
    {
        let old = self.bypass_change_detection();
        if old != value {
            value.clone_into(old);
            self.set_changed();
            true
        } else {
            false
        }
    }
}
