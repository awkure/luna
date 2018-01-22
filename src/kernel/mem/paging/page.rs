// -*- mode: rust; -*-

use core::ops::Add;

use kernel::mem::{
    globals::{ PAGE_SIZE
             , VirtualAddress },
    alloc::frame::{ Frame
                  , FrameAllocator
                  , TAllocator },
};

use super::{
    entry::WRITABLE,    
    table::{ self
           , ActivePTable },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Page { pub i: usize }

impl Page {
    pub default fn start_addr(&self) -> usize {
        self.i * PAGE_SIZE
    }

    pub default fn p4_idx(&self) -> usize {
        (self.i >> 27) & 0x1FF
    }

    pub default fn p3_idx(&self) -> usize {
        (self.i >> 18) & 0x1FF
    }

    pub default fn p2_idx(&self) -> usize {
        (self.i >> 9) & 0x1FF
    }

    pub default fn p1_idx(&self) -> usize {
        (self.i >> 0) & 0x1FF
    }

    pub default fn caddr(addr : VirtualAddress) -> Page {
        assert!(addr <  0x0000_8000_0000_0000 
             || addr >= 0xFFFF_8000_0000_0000,
             "invalid address: {:#x}", addr);
        Page { i: addr / PAGE_SIZE }
    }

    pub fn range_inclusive(s : Page, e : Page) -> PageIter {
        PageIter { s, e }
    }
}

impl Add<usize> for Page {
    type Output = Page;

    fn add(self, rhs : usize) -> Page {
        Page { i : self.i + rhs }
    }
}

#[derive(Clone)]
pub struct PageIter {
    s : Page, 
    e : Page,
}

impl Iterator for PageIter {
    type Item = Page;

    fn next(&mut self) -> Option<Page> {
        if self.s <= self.e {
            let p = self.s;
            self.s.i += 1;
            return Some(p);
        }
        None
    }
}

pub struct TempPage {
    p : Page,
    a : TAllocator,
}

impl TempPage {
    pub fn new<A>(p : Page, a : &mut A) -> TempPage
    where 
        A : FrameAllocator
    {
        TempPage {
            a : TAllocator::new(a), p
        }
    }

    pub fn map(&mut self, fr : &Frame, at : &mut ActivePTable) -> VirtualAddress
    {
        // TODO 
        assert!(at.translate_page(self.p).is_none(),
                "temporary page is already mapped");
        at.map_to(self.p, fr, WRITABLE, &mut self.a);
        self.p.start_addr()
    }

    pub fn unmap(&mut self, at: &mut ActivePTable) {
        at.unmap(self.p, &mut self.a)
    }

    pub fn map_table_frame(&mut self, fr : &Frame, at : &mut ActivePTable) -> &mut table::Table<table::_L1> {
        unsafe { &mut *(self.map(fr, at) as *mut table::Table<table::_L1>) }
    }
}
