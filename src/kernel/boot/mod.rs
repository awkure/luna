// -*- mode: rust; -*-

mod tags;

mod elf;
pub (crate) use self::elf::{
    ElfSectionHeader,
    ELF_SECTION_ALLOCATED,
    ELF_SECTION_EXECUTABLE, 
    ELF_SECTION_WRITABLE,
};

mod mmap;
pub (crate) use self::mmap::{ MemArea, MemAreaIter };

mod bi;
pub (crate) use self::bi::{ BootInfo, load };
