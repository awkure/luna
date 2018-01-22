// -*- mode: rust; -*-

#[repr(C)]
pub struct MMapTag {
    typ     : u32,
    size    : u32,
    esize   : u32,
    ever    : u32,
    f       : MemArea,
}

impl MMapTag {
    pub fn memo(&self) -> MemAreaIter {
        let ptr   = self as *const MMapTag;
        let start = (&self.f) as *const MemArea;

        MemAreaIter {
            cur     : start,
            last    : ((ptr as u32) + self.size - self.esize) as *const MemArea,
            esize   : self.esize,
        }
    }
}

#[repr(C)]
pub struct MemArea {
    pub base : u64,
    pub len  : u64,
        typ  : u32,
        _pad : u32,
}

#[derive(Clone)]
pub struct MemAreaIter {
    cur     : *const MemArea,
    last    : *const MemArea,
    esize   : u32,
}

impl Iterator for MemAreaIter {
    type Item = &'static MemArea;

    fn next(&mut self) -> Option<&'static MemArea> {
        if self.cur <= self.last {
            let area = unsafe{&*self.cur};
            
            self.cur = ((self.cur as u32) + self.esize) as *const MemArea;

            if area.typ == 1 { Some(area) } 
            else { self.next() }

        } else { None }
    }
}
