// -*- mode: rust; -*-

pub (crate) type PhysicalAddress = usize;
pub (crate) type VirtualAddress  = usize;

pub (crate) const PAGE_SIZE   : usize = 4096;
pub (crate) const ENTRY_COUNT : usize =  512;

pub (crate) const HEAP_START : usize = 0o0_000_010_000_000_000;
pub (crate) const HEAP_SIZE  : usize = 100 * 1024; 

pub (crate) const STACK_ALLOCATOR_SIZE : usize = 100;
