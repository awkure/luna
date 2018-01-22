// -*- mode: rust; -*-

pub mod globals;

pub mod control;
pub (in super::super) use self::control::init;

pub mod alloc;
mod paging;
