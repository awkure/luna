// -*- mode: rust; -*-

use core::ptr::Unique;

use super::{
    page::Page,
    table::{ self
           , Table },
    entry::{ EFlags
           , HUGE_PAGE
           , PRESENT },
}; 

use kernel::mem::{
    alloc::{ Frame
           , FrameAllocator },
    globals::{ PAGE_SIZE
             , ENTRY_COUNT
             , VirtualAddress 
             , PhysicalAddress },
};

pub struct Map { p4: Unique<Table<table::_L4>> }

impl Map {
    pub unsafe fn new() -> Map {
        Map { p4: Unique::new_unchecked(table::P4) }
    }

    pub fn p4(&self) -> &Table<table::_L4> {
        unsafe { self.p4.as_ref() }
    }

    pub fn p4_mut(&mut self) -> &mut Table<table::_L4> {
        unsafe { self.p4.as_mut() }
    }

    pub fn translate_vtop(&self, vaddr : VirtualAddress) -> Option<PhysicalAddress> {
        self.translate_page(Page::caddr(vaddr))
            .map(|f| f.i * PAGE_SIZE + vaddr % PAGE_SIZE)
    }

    pub fn translate_page(&self, p : Page) -> Option<Frame> {
        let p3 = self.p4().next_table_ref(p.p4_idx());

        let huge_page = || {
            p3.and_then(|p3| {
                let p3_e = &p3[p.p3_idx()];
                
                if let Some(sfr) = p3_e.pointed_frame() {
                    if p3_e.flags().contains(HUGE_PAGE) {
                        assert!(sfr.i % (ENTRY_COUNT * ENTRY_COUNT) == 0);
                        return Some(Frame {
                            i: sfr.i + p.p2_idx() * ENTRY_COUNT + p.p1_idx()
                        });
                    }
                }

                if let Some(p2) = p3.next_table_ref(p.p3_idx()) {
                    let p2_e = &p2[p.p2_idx()];
                
                    if let Some(sfr) = p2_e.pointed_frame() {
                        if p2_e.flags().contains(HUGE_PAGE) {
                            assert!(sfr.i % ENTRY_COUNT == 0);
                            return Some(Frame { i: sfr.i + p.p1_idx() });
                        }
                    }
                }
                None
            })
        };

        p3.and_then(|p3| p3.next_table_ref(p.p3_idx()))
          .and_then(|p2| p2.next_table_ref(p.p2_idx()))
          .and_then(|p1| p1[p.p1_idx()].pointed_frame())
          .or_else(huge_page)
    }

    pub fn map_to<A>(&mut self, p : Page, fr : &Frame, fl : EFlags, a : &mut A)
    where 
        A : FrameAllocator
    {
        let p4 = self.p4_mut();
        let p3 = p4.next_table_create(p.p4_idx(), a);
        let p2 = p3.next_table_create(p.p3_idx(), a);
        let p1 = p2.next_table_create(p.p2_idx(), a);

        assert!(p1[p.p1_idx()].is_unused());
        p1[p.p1_idx()].set(fr, fl | PRESENT);
    }

    pub fn idmap<A>(&mut self, fr : &Frame, fl : EFlags, a : &mut A)
    where 
        A : FrameAllocator
    {
        let p = Page::caddr(fr.addr_ptr());
        self.map_to(p, fr, fl, a)
    }

    pub fn map<A>(&mut self, p : Page, fl : EFlags, a : &mut A)
    where 
        A : FrameAllocator
    {
        let fr = a.alloc().expect("out of memory");
        self.map_to(p, &fr, fl, a)
    }

    pub fn unmap<A>(&mut self, p : Page, _ : &mut A)
    where 
        A : FrameAllocator
    {
        use x86_64::instructions::tlb;
        use x86_64::VirtualAddress;

        assert!(self.translate_vtop(p.start_addr()).is_some());

        let p1 = self.p4_mut()
                     .next_table_mut(p.p4_idx())
                     .and_then(|p3| p3.next_table_mut(p.p3_idx()))
                     .and_then(|p2| p2.next_table_mut(p.p2_idx()))
                     .expect("mapping code does not support huge pages");

        p1[p.p1_idx()].set_unused();
        tlb::flush(VirtualAddress(p.start_addr()));
    }
}
