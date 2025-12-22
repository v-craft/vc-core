#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![no_std]

pub use vc_cfg as cfg;
pub use vc_os as os;
pub use vc_ptr as ptr;
pub use vc_reflect as reflect;
pub use vc_task as task;
pub use vc_utils as utils;
