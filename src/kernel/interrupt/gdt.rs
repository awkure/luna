// -*- mode: rust; -*-

use x86_64::{
    PrivilegeLevel,
    structures::{ tss::TaskStateSegment
                , gdt::SegmentSelector },
    instructions::tables::{ DescriptorTablePointer
                          , lgdt },
};

use bit_field::BitField;

/// Global Descriptor Table revisited 
pub struct GDT {
    /// GDT table itself
    t : [u64 ; 8],  
    /// index of next free entry
    n : usize,      
}

impl GDT {
    /// Construct a new GDT table
    pub fn new() -> GDT {
        GDT { t : [0 ; 8], n : 1 }
    }

    /// Push the given entry to GDT table and
    /// return index to the produced free entry
    fn push(&mut self, v : u64) -> usize {
        if self.n < self.t.len() {
            let i = self.n;
            
            self.t[i] = v;
            self.n += 1;
            
            return i;
        }
        panic!("Attempt to overflow GDT");
    }

    /// Add descriptor to an existing GDT 
    pub fn add_entry(&mut self, e : &Descriptor) -> SegmentSelector {
        let i = match *e {
            Descriptor::UserSegment(v) => self.push(v),
            Descriptor::SystemSegment(l, h) => {
                let i = self.push(l);
                self.push(h);
                i
            }
        };

        SegmentSelector::new(i as u16, PrivilegeLevel::Ring0)
    }

    /// Load the GDT
    pub fn load(&'static self) {
        let p = DescriptorTablePointer {
            base  : self.t.as_ptr() as u64,
            limit : (self.t.len() * ::core::mem::size_of::<u64>() - 1) as u16,
        };

        unsafe { lgdt(&p) };
    }
}

/// Descriptor flags definition
bitflags! {
    flags DescriptorFlags : u64 {
        const CONFORMING    = 1 << 42,
        const EXECUTABLE    = 1 << 43,
        const USER_SEGMENT  = 1 << 44,
        const PRESENT       = 1 << 47,
        const LONG_MODE     = 1 << 53,
    }
}

/// Descriptor enum for handling 
/// user and system segment descriptors. 
/// Note that the system TSS descriptor
/// also contains address and a limit
/// and therefore they're 128 bits.
pub enum Descriptor {
    UserSegment(u64),
    SystemSegment(u64, u64),
}

impl Descriptor {
    /// Returns user code segment with stadnard flags provided.
    pub fn kernel_code_segment() -> Descriptor {
        let flags = EXECUTABLE | USER_SEGMENT | PRESENT | LONG_MODE;
        Descriptor::UserSegment(flags.bits())
    }


    /// Create TSS mode segment.
    ///
    /// TSS descriptor is a system segment corresponded to the following table
    ///
    /// | Bit      |  Name               | Describtion                                  |
    /// | -------- | ------------------- | -------------------------------------------- |
    /// | 0..15    | Limit 0..15         | Memory range                                 |
    /// | 16..39   | Base Address 0..23  | Linear address where the memory range begins |
    /// | 40..43   | Descriptor Type     | Here must be `0b1001` for TSS                |
    /// | 44       | Storage Segment     | Must be equal to zero                        |
    /// | 45..46   | Privilege Level     | Ring level : 0 -- kernel, 3 -- user          |
    /// | 47       | Present             | 1 for valid selectors                        |
    /// | 48..51   | Limit 16..19        | High part of the limit                       |
    /// | 52       | Free                | Freely available                             |
    /// | 53..54   | Ignored             |                                              |
    /// | 55       | Granularity         | Limit the page/byte number                   |
    /// | 56..64   | Base Address 24..31 | Higher part of the Base Address              |
    /// | 64..95   | Base Address 32..63 | The last four bytes of the Base Address      |
    /// | 96..104  | Ignored             |                                              |
    /// | 104..108 | ZERO                |                                              |
    /// | 108..127 | Ignored             |                                              |
    pub fn tss_segment(tss : &'static TaskStateSegment) -> Descriptor {
        let ptr = tss as *const _ as u64;

        let mut l = PRESENT.bits();

        l.set_bits(0..16, (::core::mem::size_of::<TaskStateSegment>() -1) as u64);

        l.set_bits(16..40, ptr.get_bits(0..24));
        l.set_bits(56..64, ptr.get_bits(24..32));

        // 80386-TSS, 32 bit 
        l.set_bits(40..44, 0b1001);

        let mut h = 0;
        h.set_bits(0..32, ptr.get_bits(32..64));

        Descriptor::SystemSegment(l,h)
    }
}

