// -*- mode: rust; -*-

pub mod entry;

pub mod table;
pub (super) use self::table::{ ActivePTable, InactivePTable };

pub mod page;
pub (super) use self::page::{ Page, PageIter };

mod map;
