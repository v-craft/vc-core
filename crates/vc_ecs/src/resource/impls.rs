#![allow(clippy::missing_safety_doc, reason = "todo")]

use crate::clone::CloneBehavior;

pub unsafe trait Resource: Sized + 'static {
    const IS_SEND: bool;
    const MUTABLE: bool;
    const CLONE_BEHAVIOR: CloneBehavior = CloneBehavior::Refuse;
}
