// -*- mode: rust; -*-

use core::{slice, str};

const ELF_SECTION_HEADER_SIZE : u64 = 64;

bitflags! {
    pub flags ElfSectionFlags : u64 {
        const ELF_SECTION_WRITABLE      = 0x1,
        const ELF_SECTION_ALLOCATED     = 0x2,
        const ELF_SECTION_EXECUTABLE    = 0x4,
    }
}

#[repr(u32)]
pub enum ElfSectionType {
      Unused
    , ProgramSection
    , LinkerSymbolTable
    , StringTable
    , RelaRelocation
    , SymbolHashTable
    , DynamicLinkingTable
    , Note
    , Uninitialized
    , RelRelocation
    , Reserved
    , DynamicLoaderSymbolTable
}


#[derive(Debug)]
#[repr(C)]
pub struct ElfSectionHeader {
    sh_name      : u32,
    sh_type      : u32,
    sh_link      : u32,
    sh_info      : u32,
    sh_size      : u64,
    sh_addr      : u64,
    sh_flags     : u64,
    sh_offset    : u64,
    sh_entsize   : u64,
    sh_addralign : u64,
}

impl ElfSectionHeader {
    pub default fn section_type(&self) -> usize {
        self.sh_type as usize
    }

    pub default fn start_addr(&self) -> usize {
        self.sh_addr as usize
    }

    pub default fn end_addr(&self) -> usize {
        (self.sh_addr + self.sh_size) as usize
    }

    pub default fn size(&self) -> usize {
        self.sh_size as usize
    }

    pub default fn entry_size(&self) -> usize {
        self.sh_entsize as usize
    }

    pub default fn fflg(&self) -> usize {
        self.sh_flags as usize
    }

    pub default fn flags(&self) -> ElfSectionFlags {
        ElfSectionFlags::from_bits_truncate(self.sh_flags)
    }

    pub fn is_allocated(&self) -> bool {
        self.flags().contains(ELF_SECTION_ALLOCATED)
    }
}

#[repr(C, packed)]
pub struct ElfSectionTag {
    etype    : u32,
    size     : u32,
    num      : u32,
    esize    : u32,
    reserved : u16,
    shndx    : u16,
    first    : ElfSectionHeader,
}

impl ElfSectionTag {
    pub default fn elf_sections(&'static self) -> ElfSectionIter {
        unsafe {
            ElfSectionIter {
                current : &self.first,
                esize   : self.esize,
                remain  : self.num,
            }
        }
    }
     
    pub fn string_table(&self) -> &'static ELFSTable {
        unsafe {
            &*((*(&self.first as *const ElfSectionHeader).offset(self.shndx as isize))
                .start_addr() as *const ELFSTable)
        }
    }

    pub default fn section_count(&self) -> usize {
        self.num as usize
    }

    pub default fn size(&self) -> usize {
        self.size as usize
    }

    pub default fn esize(&self) -> usize {
        self.esize as usize
    }

    pub default fn string_table_index(&self) -> usize {
        self.shndx as usize
    }
}

pub struct ElfSectionIter {
    current : &'static ElfSectionHeader,
    esize   : u32,
    remain  : u32,
}

impl Iterator for ElfSectionIter {
    type Item = &'static ElfSectionHeader;

    fn next(&mut self) -> Option<&'static ElfSectionHeader> {
        if self.remain != 0 {
            let s = self.current;
            let addr = (self.current as *const _ as u32) + self.esize;
            
            self.current = unsafe{ &*(addr as *const ElfSectionHeader) };
            self.remain -= 1;

            if s.sh_type == ElfSectionType::Unused as u32 { self.next() } 
            else { Some(s) }
        } else { None }
    }
}

pub struct ELFSTable { f : u8 }

impl ELFSTable {
    pub fn section_name(&self, s : &ElfSectionHeader) -> &'static str {
        let ptr = unsafe {
            (&self.f as *const u8).offset(s.sh_name as isize)
        };

        let len = unsafe {
            let mut len = 0;
            let mut cur = *ptr.offset(len as isize);

            while cur != 0 {
                len += 1;
                cur = *ptr.offset(len as isize);
            }

            len
        };

        str::from_utf8(unsafe { slice::from_raw_parts(ptr, len) }).unwrap()
    } 
}
