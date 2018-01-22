// -*- mode: rust; -*-

#[repr(packed)]
pub struct ModuleTag {
    typ     : u32,
    size    : u32,
    start   : u32,
    end     : u32,
    byte    : u8,
}

impl ModuleTag {
    pub fn name(&self) -> &str {
        use core::{ mem, str, slice };

        let strlen = self.size as usize - mem::size_of::<ModuleTag>();

        unsafe {
            str::from_utf8_unchecked(slice::from_raw_parts(&self.byte as *const u8, strlen))
        }
    }

    pub fn start_addr(&self) -> u32 {
        self.start
    }

    pub fn end_addr(&self) -> u32 {
        self.end
    }
}


#[repr(packed)]
pub struct BootNameTag {
    typ  : u32,
    size : u32,
    name : u8,
}

impl BootNameTag {
    pub fn name(&self) -> &str {
        unsafe { ::core::str::from_utf8_unchecked(::core::slice::from_raw_parts((&self.name) as *const u8, self.size as usize - 8)) }
    }
}
