// -*- mode: rust; -*-

use x86_64::{
    instructions::tlb,
    registers::control_regs,
};

use core::{
    marker::PhantomData,
    ops::{ Index
         , IndexMut
         , Deref
         , DerefMut },
};

use kernel::mem::{
    globals::ENTRY_COUNT,
    alloc::{ FrameAllocator
           , Frame },
}; 

use super::{
    page::TempPage,
    map::Map,
    entry::{ Entry
           , HUGE_PAGE
           , PRESENT
           , WRITABLE },
}; 

pub const P4: *mut Table<_L4> = 0xFFFF_FFFF_FFFF_F000 as *mut _;

pub enum _L4 {}
pub enum _L3 {}
pub enum _L2 {}
pub enum _L1 {}

pub trait _TL {}

impl _TL for _L4 {}
impl _TL for _L3 {}
impl _TL for _L2 {}
impl _TL for _L1 {}

pub trait HLayer: _TL { type NL: _TL; }

impl HLayer for _L4 { type NL = _L3; }
impl HLayer for _L3 { type NL = _L2; }
impl HLayer for _L2 { type NL = _L1; }

pub struct Table<L: _TL> {
    es : [Entry ; ENTRY_COUNT],
    lv : PhantomData<L>,
}

impl<L> Table<L> where L: _TL {
    pub fn zero(&mut self) {
        self.es.iter_mut().for_each(|e| e.set_unused());
    }
}

impl<L> Table<L> 
where 
    L : HLayer 
{
    fn next_taddr(&self, i : usize) -> Option<usize> {
        let eflags = self[i].flags();
        if eflags.contains(PRESENT) && !eflags.contains(HUGE_PAGE) {
            let taddr = self as *const _ as usize;
            return Some((taddr << 9) | (i << 12));
        }
        None
    }

    pub default fn next_table_ref(&self, i : usize) -> Option<&Table<L::NL>> {
        self.next_taddr(i)
            .map(|addr| unsafe { &*(addr as *const _) })
    }

    pub default fn next_table_mut(&mut self, i : usize) -> Option<&mut Table<L::NL>> {
        self.next_taddr(i)
            .map(|addr| unsafe { &mut *(addr as *mut _) })
    }

    pub default fn next_table_create<A>(&mut self, i : usize, a : &mut A) -> &mut Table<L::NL>
    where 
        A : FrameAllocator
    {
        if self.next_table_ref(i).is_none() {
            assert!(!self.es[i].flags().contains(HUGE_PAGE), "huge page mapping is unsupported (TODO)");
            let fr = a.alloc().expect("no frames available");
            self.es[i].set(&fr, PRESENT | WRITABLE);
            self.next_table_mut(i).unwrap().zero();
        }
        self.next_table_mut(i).unwrap()
    }
}

impl<L> Index<usize> for Table<L> 
where 
    L : _TL 
{
    type Output = Entry;

    fn index(&self, i : usize) -> &Entry {
        &self.es[i]
    }
}

impl<L> IndexMut<usize> for Table<L> 
where
    L : _TL 
{
    fn index_mut(&mut self, i : usize) -> &mut Entry {
        &mut self.es[i]
    }
}

pub struct ActivePTable {
    map : Map,
}

impl Deref for ActivePTable {
    type Target = Map;

    fn deref(&self) -> &Map {
        &self.map
    }
}

impl DerefMut for ActivePTable {
    fn deref_mut(&mut self) -> &mut Map {
        &mut self.map
    }
}

impl ActivePTable {
    pub unsafe fn new() -> ActivePTable {
        ActivePTable { map: Map::new() }
    }

    pub fn with<F>(&mut self, t : &mut InactivePTable, p : &mut TempPage, f : F)
    where 
        F : FnOnce(&mut Map)
    {
        {
            let buf = Frame::caddr(control_regs::cr3().0 as usize);

            let p4_t = p.map_table_frame(&buf, self);

            self.p4_mut()[511].set(&t.p4_frame.clone(), PRESENT | WRITABLE);
            tlb::flush_all();

            f(self);

            p4_t[511].set(&buf, PRESENT | WRITABLE);
            tlb::flush_all();
        }

        p.unmap(self);
    }

    pub fn switch(&mut self, new_t : &InactivePTable) -> InactivePTable {
        use x86_64::PhysicalAddress;

        let old_t = InactivePTable {
            p4_frame : Frame::caddr(control_regs::cr3().0 as usize)
        };

        unsafe {
            let paddr = PhysicalAddress(new_t.p4_frame.addr_ptr() as u64);
            control_regs::cr3_write(paddr);
        }

        old_t
    }
}

pub struct InactivePTable {
    pub p4_frame : Frame
}

impl InactivePTable {
    pub fn new(fr : Frame, at : &mut ActivePTable, p : &mut TempPage) -> InactivePTable {
        {
            let t = p.map_table_frame(&fr,at);
            t.zero();
            t[511].set(&fr.clone(), PRESENT | WRITABLE);
        }
        
        p.unmap(at);

        InactivePTable { p4_frame : fr }
    }
}
