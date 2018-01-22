// -*- mode: rust; -*-

use core::{
    ptr::Unique,
    default::Default,
    mem::{ self
         , size_of },
};

use alloc::allocator::{Layout, AllocErr};

use super::heap::align_up;

/// A separate struct needed for some pointer magic which contains basic 
/// information about address and the size of the free memory block.
#[derive(Debug, Clone, Copy)]
struct HoleConv {
    a : usize,
    s : usize,
}

/// Allocation sruct which contains address and 
/// size of the allocation, front and back padding.
struct Alloc { 
    info : HoleConv,
    fr_p : Option<HoleConv>,
    ba_p : Option<HoleConv>,
}

/// Basic Hole implementation which contains its size and a unique
/// pointer to the next hole thus forming a singly-linked list.
#[cfg(not(test))]
#[derive(Default)]
pub struct Hole {
    s : usize,
    n : Option<Unique<Hole>>,
}

impl Hole {
    /// Makes a converted hole with and address inside.
    default fn info(&self) -> HoleConv {
        HoleConv {
            a : self as *const _ as usize,
            s : self.s,
        }
    }

    /// Returns a reference to the next hole.
    fn next_unwrap(&mut self) -> &mut Hole {
        unsafe { self.n.as_mut().unwrap().as_mut() }
    }
}

/// Sorted list of holes
pub struct Holes {
    f : Hole
}

impl Holes {
    /// Return a blank list of holes
    pub default const fn blank() -> Holes {
        Holes {
            f : Hole { s : 0 , n : None }
        }
    }

    /// Create a list of holes that contains the given hole.
    /// It's unsafe since it uses direct addressing to memory
    /// block which can be used somewhere else.
    pub unsafe fn new(h_addr : usize, h_size : usize) -> Holes {
        assert!(size_of::<Hole>() == Self::min_size());

        let p = h_addr as *mut Hole;

        mem::replace(&mut *p, Hole { s : h_size, n : None });

        Holes {
            f : Hole { n : Some(Unique::new_unchecked(p)), .. Default::default() }
        }
    }

    /// Return the minimal allocation size
    pub default fn min_size() -> usize {
        size_of::<usize>() * 2
    }

    /// Searches for the bin enough hole to return the address of 
    /// the block required for memory allocation purposes.
    pub fn alloc_first_fit(&mut self, l : Layout) -> Result<*mut u8, AllocErr> {
        assert!(l.size() >= Self::min_size());

        alloc_first_fit(&mut self.f, l).map(|a| {
            if let Some(p) = a.fr_p { dealloc(&mut self.f, p.a, p.s); }
            if let Some(p) = a.ba_p { dealloc(&mut self.f, p.a, p.s); }
            a.info.a as *mut u8
        })
    } 

    /// Frees the allocation given by `p` and `l`. 
    /// UB may occur for invalid arguments.
    pub unsafe fn dealloc(&mut self, p : *mut u8, l : &Layout) {
        dealloc(&mut self.f, p as usize, l.size())
    }
}

fn split_hole(h : HoleConv, l : &Layout) -> Option<Alloc> {
    let req_s = l.size();
    let req_a = l.align();

    let (aligned_addr, fr_p) = if h.a == align_up(h.a, req_a) { 
        (h.a, None) 
    } else {
        let aligned_addr = align_up(h.a + Holes::min_size(), req_a);
        (
            aligned_addr,
            Some(HoleConv { a : h.a , s : aligned_addr - h.a })
        )
    };

    let aligned_hole = {
        if aligned_addr + req_s > h.a + h.s { return None; }
        HoleConv {
            a : aligned_addr,
            s : h.s - (aligned_addr - h.a),
        }
    };

    let ba_p = if aligned_hole.s == req_s {
        None
    } else if aligned_hole.s - req_s < Holes::min_size() {
        return None;
    } else {
        Some(HoleConv {
            a : aligned_hole.a + req_s,
            s : aligned_hole.s - req_s,
        })
    };

    Some(Alloc {
        info : HoleConv {
            a : aligned_hole.a,
            s : req_s,
        }, fr_p, ba_p
    })
}

fn move_id<T>(a : T) -> T { a }

/// Searches the list for a big enough hole by using "first fit" strategy,
/// so it doesn't need to search the whole list for choosing the better one.
fn alloc_first_fit(mut prev : &mut Hole, l : Layout) -> Result<Alloc, AllocErr> {
    loop {
        match prev.n.as_mut().and_then(|cur| { 
            split_hole(unsafe { cur.as_ref() }.info(), &l) 
        }) {
            Some(allocation) => {
                prev.n = prev.next_unwrap().n.take();
                return Ok(allocation);
            }
            None if prev.n.is_some() => {
                prev = move_id(prev).next_unwrap();
            }
            None => {
                return Err(AllocErr::Exhausted { request : l });
            }
        }
    }
}

/// Deallocate the allocation provided by `addr` and `size`.
fn dealloc(mut hole : &mut Hole, addr : usize, mut size : usize) {
    loop {
        assert!(size >= Holes::min_size());

        let hole_addr = if hole.s == 0 { 0 } else { hole as *mut _ as usize };

        assert!(hole_addr + hole.s <= addr, "double free error");

        match hole.n.as_ref().map(|n| unsafe { n.as_ref().info() }) { // possible bug here
            Some(next) if hole_addr + hole.s == addr && addr + size == next.a => {
                hole.s += size + next.s; 
                hole.n  = hole.next_unwrap().n.take();
            }

            _ if hole_addr + hole.s == addr => {
                hole.s += size;
            }

            Some(next) if addr + size == next.a => {
                hole.n = hole.next_unwrap().n.take();
                size += next.s;
                continue;
            }

            Some(next) if next.a <= addr => {
                hole = move_id(hole).next_unwrap();
                continue;
            }

            _ => {
                let new_hole = Hole {
                    s : size,
                    n : hole.n.take(),
                };

                let ptr = addr as *mut Hole;

                mem::replace(unsafe { &mut *ptr }, new_hole);

                hole.n = Some(unsafe { Unique::new_unchecked(ptr) });
            }
        } 
        break;
    }
}
