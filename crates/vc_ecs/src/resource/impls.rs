#![allow(clippy::missing_safety_doc, reason = "todo")]

use crate::utils::{Cloner, Dropper};

pub unsafe trait Resource: Sized + 'static {
    const MUTABLE: bool = true;
    const CLONER: Option<Cloner> = None;
    const DROPPER: Option<Dropper> = None;
}
