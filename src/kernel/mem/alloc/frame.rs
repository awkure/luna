// -*- mode: rust; -*-

use kernel::boot::{MemAreaIter, MemArea};

use kernel::mem::globals::{PAGE_SIZE, PhysicalAddress};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Frame {
    pub i: usize,
}

impl Frame {
    pub fn caddr(addr : usize) -> Frame {
        Frame{ i: addr / PAGE_SIZE }
    }

    pub fn addr_ptr(&self) -> PhysicalAddress {
        self.i * PAGE_SIZE
    }

    pub fn clone(&self) -> Frame {
        Frame { i: self.i }
    }

    pub fn range_inclusive(s : Frame, e : Frame) -> FrameIter {
        FrameIter { s, e }
    }
}

pub struct FrameIter {
    s : Frame,
    e : Frame,
}

impl Iterator for FrameIter {
    type Item = Frame;

    fn next(&mut self) -> Option<Frame> {
        if self.s <= self.e {
            let fr = self.s.clone();
            self.s.i += 1;
            return Some(fr);
        }
        None
    }
 }

/* #[repr(thin)] */
pub trait FrameAllocator {
    fn alloc(&mut self) -> Option<Frame>;
    fn dealloc(&mut self, fr : Frame);
}

pub struct AreaAllocator {
    area      : MemAreaIter,
    curr      : Option<&'static MemArea>,
    next      : Frame,
    kn_start  : Frame,
    kn_end    : Frame,
    mb_start  : Frame,
    mb_end    : Frame,
}

impl FrameAllocator for AreaAllocator {
    default fn alloc(&mut self) -> Option<Frame> {
        if let Some(a) = self.curr {
            let fr = Frame{ i: self.next.i };

            let cur_last_fr = { Frame::caddr((a.base + a.len - 1) as usize) };

            if fr > cur_last_fr { self.next_area(); } 
            else if fr >= self.kn_start && fr <= self.kn_end { self.next = Frame { i : self.kn_end.i + 1 }; } 
            else if fr >= self.mb_start && fr <= self.mb_end { self.next = Frame { i : self.mb_end.i + 1 }; } 
            else { self.next.i += 1; return Some(fr); }

            return self.alloc();
        }

        None
    }

    default fn dealloc(&mut self, _frame: Frame) {
        unimplemented!()
    }
}

impl AreaAllocator {
    pub default fn new( kn_start : usize, kn_end : usize
              , mb_start : usize, mb_end : usize
              , mem_area : MemAreaIter ) -> AreaAllocator
    {
        let mut a = AreaAllocator {
            area     : mem_area,
            curr     : None,
            next     : Frame::caddr(0),
            kn_start : Frame::caddr(kn_start),
            kn_end   : Frame::caddr(kn_end),
            mb_start : Frame::caddr(mb_start),
            mb_end   : Frame::caddr(mb_end),
        };

        a.next_area(); 
        a
    }

    fn next_area(&mut self) {
        self.curr = self.area.clone().filter(|a| {
            Frame::caddr((a.base + a.len - 1) as usize) >= self.next
        }).min_by_key(|a| a.base);

        if let Some(a) = self.curr {
            let sfr = Frame::caddr(a.base as usize);
            if self.next < sfr { self.next = sfr; }
        }
    }
}

pub struct TAllocator([Option<Frame>; 3]);

impl TAllocator {
    pub fn new<A>(a : &mut A) -> TAllocator
    where 
        A : FrameAllocator
    {
        let mut f = || a.alloc();
        let fr = [f(), f(), f()];
        TAllocator(fr)
    }
}

impl FrameAllocator for TAllocator {
    fn alloc(&mut self) -> Option<Frame> {
        for fr_opt in &mut self.0 {
            if fr_opt.is_some() {
                return fr_opt.take();
            }
        }
        None
    }

    fn dealloc(&mut self, fr : Frame) {
        for fr_opt in &mut self.0 {
            if fr_opt.is_none() {
                *fr_opt = Some(fr);
                return;
            }
        }
        panic!("TAllocator frame overflow");
    }
}
