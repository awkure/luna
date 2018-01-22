// -*- mode: rust; -*-

use kernel::boot::ElfSectionHeader;

use kernel::mem::alloc::Frame;

#[derive(Default)]
pub struct Entry(u64);

impl Entry {
    pub default fn is_unused(&self) -> bool {
        self.0 == 0
    }

    pub default fn set_unused(&mut self) {
        self.0 = 0;
    }

    pub default fn flags(&self) -> EFlags {
        EFlags::from_bits_truncate(self.0)
    }

    pub fn pointed_frame(&self) -> Option<Frame> {
        if self.flags().contains(PRESENT) {
            return Some(Frame::caddr(self.0 as usize & 0x000F_FFFF_FFFF_F000));
        }
        None
    }

    pub fn set(&mut self, fr : &Frame, fl : EFlags) {
        assert!(fr.addr_ptr() & !0x000F_FFFF_FFFF_F000 == 0);
        self.0 = (fr.addr_ptr() as u64) | fl.bits();
    }
}

bitflags! {
    pub flags EFlags : u64 {
        const PRESENT           = 1 <<  0,
        const WRITABLE          = 1 <<  1,
        const USER_ACCESSIBLE   = 1 <<  2,
        const WRITE_THROUGH     = 1 <<  3,
        const NO_CACHE          = 1 <<  4,
        const ACCESSED          = 1 <<  5,
        const DIRTY             = 1 <<  6,
        const HUGE_PAGE         = 1 <<  7,
        const GLOBAL            = 1 <<  8,
        const NO_EXECUTE        = 1 << 63,
    }
}

impl EFlags {
    pub fn from_elf_section_flags(section: &ElfSectionHeader) -> EFlags {
        use kernel::boot::{ ELF_SECTION_ALLOCATED
                          , ELF_SECTION_WRITABLE
                          , ELF_SECTION_EXECUTABLE 
                          };

        let mut flags = EFlags::empty();

        if  section.flags().contains(ELF_SECTION_ALLOCATED)  { flags |= PRESENT    ; }
        if  section.flags().contains(ELF_SECTION_WRITABLE)   { flags |= WRITABLE   ; }
        if !section.flags().contains(ELF_SECTION_EXECUTABLE) { flags |= NO_EXECUTE ; }

        flags
    }
}
