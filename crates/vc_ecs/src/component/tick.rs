use crate::tick::Tick;

// -----------------------------------------------------------------------------
// ComponentTickRef

#[derive(Debug, Clone)]
pub(crate) struct ComponentTicksRef<'w> {
    pub(crate) added: &'w Tick,
    pub(crate) changed: &'w Tick,
    pub(crate) last_run: Tick,
    pub(crate) this_run: Tick,
}

// -----------------------------------------------------------------------------
// ComponentTicksMut

#[derive(Debug)]
pub(crate) struct ComponentTicksMut<'w> {
    pub(crate) added: &'w mut Tick,
    pub(crate) changed: &'w mut Tick,
    pub(crate) last_run: Tick,
    pub(crate) this_run: Tick,
}

impl<'w> From<ComponentTicksMut<'w>> for ComponentTicksRef<'w> {
    #[inline(always)]
    fn from(this: ComponentTicksMut<'w>) -> Self {
        ComponentTicksRef {
            added: this.added,
            changed: this.changed,
            last_run: this.last_run,
            this_run: this.this_run,
        }
    }
}

// -----------------------------------------------------------------------------
// ComponentTicksSliceRef

#[derive(Debug, Clone)]
pub(crate) struct ComponentTicksSliceRef<'w> {
    pub(crate) added: &'w [Tick],
    pub(crate) changed: &'w [Tick],
    pub(crate) last_run: Tick,
    pub(crate) this_run: Tick,
}

// -----------------------------------------------------------------------------
// ComponentTicksSliceMut

#[derive(Debug)]
pub(crate) struct ComponentTicksSliceMut<'w> {
    pub(crate) added: &'w mut [Tick],
    pub(crate) changed: &'w mut [Tick],
    pub(crate) last_run: Tick,
    pub(crate) this_run: Tick,
}

impl<'w> From<ComponentTicksSliceMut<'w>> for ComponentTicksSliceRef<'w> {
    #[inline(always)]
    fn from(this: ComponentTicksSliceMut<'w>) -> Self {
        ComponentTicksSliceRef {
            added: this.added,
            changed: this.changed,
            last_run: this.last_run,
            this_run: this.this_run,
        }
    }
}
