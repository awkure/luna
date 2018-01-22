// -*- mode: rust; -*-

use super::elf::ElfSectionTag;
use super::mmap::MMapTag;
use super::tags::{ ModuleTag, BootNameTag };

#[repr(C)]
struct Tag {
    typ  : u32,
    size : u32,
}

struct TagIter {
    cur : *const Tag
}

impl Iterator for TagIter {
    type Item = &'static Tag;

    fn next(&mut self) -> Option<&'static Tag> {
        match unsafe{&*self.cur} {
            &Tag{typ:0,size:8} => None,
            t => {
                let mut addr = self.cur as usize;
                
                addr += t.size as usize;
                addr = ((addr-1) & !0x7) + 0x8;
                
                self.cur = addr as *const _;

                Some(t)
            },
        }
    }
}

#[repr(C)]
pub struct BootInfo {
    pub total_size : u32,
    _pad : u32,
    ft : Tag,
}

impl BootInfo {
    fn has_valid_end_tag(&self) -> bool {
        const END_TAG: Tag = Tag{typ:0, size:8};

        let self_ptr = self as *const _;
        let end_tag_addr = self_ptr as usize + (self.total_size - END_TAG.size) as usize;
        let end_tag = unsafe{&*(end_tag_addr as *const Tag)};

        end_tag.typ == END_TAG.typ && end_tag.size == END_TAG.size
    }

    pub default fn start_addr(&self) -> usize {
        self as *const _ as usize
    }

    pub default fn end_addr(&self) -> usize {
        self.start_addr() + self.total_size as usize
    }

    pub default fn elf_sections_tag(&self) -> Option<&'static ElfSectionTag> {
        self.get_tag(9).map(|t| unsafe {&*(t as *const Tag as *const ElfSectionTag)})
    }

    pub default fn memory_map_tag(&self) -> Option<&'static MMapTag> {
        self.get_tag(6).map(|t| unsafe{&*(t as *const Tag as *const MMapTag)})
    }

    pub default fn module_tag(&self) -> Option<&'static ModuleTag> {
        self.get_tag(3).map(|t| unsafe{&*(t as *const Tag as *const ModuleTag)})
    }

    pub default fn boot_loader_name_tag(&self) -> Option<&'static BootNameTag> {
        self.get_tag(2).map(|t| unsafe{&*(t as *const Tag as *const BootNameTag)})
    }

    fn get_tag(&self, typ : u32) -> Option<&'static Tag> {
        self.tags().find(|t| t.typ == typ)
    }

    fn tags(&self) -> TagIter {
        TagIter{ cur: &self.ft as *const _ }
    }
}

pub unsafe fn load(addr : usize) -> &'static BootInfo {
    let mb = &*(addr as *const BootInfo);
    
    assert!(mb.has_valid_end_tag());
    
    mb
}
