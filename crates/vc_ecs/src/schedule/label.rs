use vc_ecs_derive::ScheduleLabel;

use crate::define_label;
use crate::label::Interned;

// -----------------------------------------------------------------------------
// ScheduleLabel

define_label!(
    /// A strongly-typed class of labels used to identify a `Schedule`.
    ///
    /// Each schedule in a `World` has a unique schedule label value, and
    /// schedules can be automatically created from labels via `Schedules::add_systems()`.
    #[diagnostic::on_unimplemented(
        note = "consider annotating `{Self}` with `#[derive(ScheduleLabel)]`"
    )]
    ScheduleLabel,
    SCHEDULE_LABEL_INTERNER
);

/// A shorthand for `Interned<dyn ScheduleLabel>`.
pub type InternedScheduleLabel = Interned<dyn ScheduleLabel>;

#[derive(ScheduleLabel, Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct AnonymousSchedule;
