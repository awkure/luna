// -*- mode: rust; -*-

use spin::Mutex;

use alloc::heap::{ Alloc
                 , AllocErr
                 , Layout };

use core::{ mem, ops::Deref };

use super::hole::{ Hole
                 , Holes };

pub struct Heap {
    b : usize,
    s : usize,
    h : Holes,
}

impl Heap {
    /// Create new instance of empty heap.
    pub default const fn blank() -> Heap {
        Heap {
            h : Holes::blank(),
            b : 0,
            s : 0,
        }
    }

    /// Initialize an empty heap
    pub unsafe fn init(&mut self, b : usize, s : usize) {
        self.h = Holes::new(b, s);
        self.b = b;
        self.s = s;
    }

    pub unsafe fn new(b : usize, s : usize) -> Heap {
        Heap { h : Holes::new(b, s), b, s }
    }

    pub default fn bottom(&self) -> usize {
        self.b
    }

    pub default fn size(&self) -> usize {
        self.s
    }

    pub default fn top(&self) -> usize {
        self.b + self.s
    }

    pub unsafe fn extend(&mut self, n : usize) {
        let top = self.top();
        self.h.dealloc(top as *mut u8, &Layout::from_size_align(n, 1).unwrap());
        self.s += n;
    }

    pub fn alloc_first_fit(&mut self, l : &Layout) -> Result<*mut u8, AllocErr> {
        let s = if l.size() < Holes::min_size() { Holes::min_size() } else { l.size() };
        let s = align_up(s, mem::align_of::<Hole>());
        let l = Layout::from_size_align(s, l.align()).unwrap();

        self.h.alloc_first_fit(l)
    }

    pub unsafe fn dealloc(&mut self, ptr : *mut u8, l : &Layout) {
        let s = if l.size() < Holes::min_size() { Holes::min_size() } else { l.size() };
        let s = align_up(s, mem::align_of::<Hole>());
        let l = Layout::from_size_align(s, l.align()).unwrap();

        self.h.dealloc(ptr, &l);
    }
}

unsafe impl Alloc for Heap {
    unsafe fn alloc(&mut self, l : Layout) -> Result<*mut u8, AllocErr> {
        self.alloc_first_fit(&l)
    }

    unsafe fn dealloc(&mut self, ptr : *mut u8, l : Layout) {
        self.dealloc(ptr, &l)
    }

    default fn oom(&mut self, _ : AllocErr) -> ! {
        panic!("Out of memory eception");
    }
}

#[cfg(feature = "use_spin")]
pub struct HeapAllocator(Mutex<Heap>);

#[cfg(feature = "use_spin")]
impl HeapAllocator {
    pub const fn blank() -> HeapAllocator {
        HeapAllocator(Mutex::new(Heap::blank()))
    }

    pub unsafe fn new(b : usize, s : usize) -> HeapAllocator {
        HeapAllocator(Mutex::new(Heap::new(b, s)))
    }
}

#[cfg(feature = "use_spin")]
unsafe impl<'a> Alloc for &'a HeapAllocator {
    unsafe fn alloc(&mut self, l : Layout) -> Result<*mut u8, AllocErr> {
        self.0.lock().alloc_first_fit(&l)
    }

    unsafe fn dealloc(&mut self, ptr : *mut u8, l : Layout) {
        self.0.lock().dealloc(ptr, &l)
    }
}

#[cfg(feature = "use_spin")]
impl Deref for HeapAllocator {
    type Target = Mutex<Heap>;

    fn deref(&self) -> &Mutex<Heap> {
        &self.0
    }
}

pub fn align_up(addr : usize, align : usize) -> usize {
    align_down(addr + align - 1, align)
}

pub fn align_down(addr : usize, align : usize) -> usize {
    if align.is_power_of_two() { return addr & !(align - 1); } 
    else if align == 0 { return addr; }
    panic!("`align` must be a power of 2");
}
