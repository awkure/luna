// -*- mode: rust; -*-

use kernel::mem::{
    paging::{ self
            , Page
            , PageIter
            , ActivePTable },
    globals::PAGE_SIZE,
};

use super::FrameAllocator;

#[derive(Debug)]
pub struct Stack {
    t : usize,
    b : usize,
}

impl Stack {
    fn new(t : usize, b : usize) -> Stack {
        assert!(t > b);
        Stack { t, b }
    }

    pub default fn top(&self) -> usize {
        self.t
    }

    pub default fn bottom(&self) -> usize {
        self.b
    }
}

pub struct StackAllocator {
    r : PageIter
}

impl StackAllocator {
    pub fn new(r : PageIter) -> StackAllocator {
        StackAllocator { r }
    }

    pub default fn alloc<A : FrameAllocator>(
          &mut self 
        , at   : &mut ActivePTable
        , fr_a : &mut A
        , size : usize
        ) -> Option<Stack> {
        
        if size == 0 { return None; }

        let mut range = self.r.clone();

        let guard_page = range.next();
        
        let stack_s = range.next();
        let stack_e = if size == 0 { stack_s } else { range.nth(size - 2) };

        match (guard_page, stack_s, stack_e) {
            (Some(_), Some(s), Some(e)) => {
                self.r = range;

                Page::range_inclusive(s,e).for_each(|p| at.map(p, paging::entry::WRITABLE, fr_a));

                let stack_top = e.start_addr() + PAGE_SIZE;
                Some(Stack::new(stack_top, s.start_addr()))
            }
            _ => None,
        }
    }
}
