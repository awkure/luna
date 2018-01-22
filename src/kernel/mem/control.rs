// -*- mode: rust; -*-

use kernel::boot::BootInfo;

use kernel::mem::{
    globals::{ PAGE_SIZE
             , HEAP_START 
             , HEAP_SIZE 
             , STACK_ALLOCATOR_SIZE },
    alloc::frame::{ AreaAllocator
                  , Frame
                  , FrameAllocator },
};

use super::{
    paging::{
        table::{ self
               , ActivePTable
               , InactivePTable },
        entry::{ EFlags 
               , PRESENT
               , WRITABLE },
        page::{ Page 
              , TempPage } },
    alloc::{ stack
           , frame },
};

pub struct MemoryController {
    _at   : table::ActivePTable,
    _fr_a : frame::AreaAllocator,
    _st_a : stack::StackAllocator,
}

impl MemoryController {
    pub default fn alloc(&mut self, size : usize) -> Option<stack::Stack> {
        let &mut MemoryController { ref mut _at
                                  , ref mut _fr_a
                                  , ref mut _st_a } = self;
        _st_a.alloc(_at, _fr_a, size)
    }
}

pub fn kernel_remap<A>(a : &mut A, b : &BootInfo) -> ActivePTable 
where 
    A : FrameAllocator
{
    let mut p = TempPage::new(Page { i : 0xCACABA }, a);

    let mut at = unsafe { ActivePTable::new() };
    let mut new_t = {
        let fr = a.alloc().expect("no more frames available");
        InactivePTable::new(fr, &mut at, &mut p)
    };

    at.with(&mut new_t, &mut p, |map| {
        let elf_sections = b.elf_sections_tag().expect("Memory map tag required");

        println!("\nMapping");
        println!("\taddr {:>9}", "size");
        
        for s in elf_sections.elf_sections() {
            if !s.is_allocated() { continue; }

            assert!(s.start_addr() % PAGE_SIZE == 0, "sections need to be aligned");

            println!("\t{:#x} {{{:#x}}}", s.start_addr(), s.size());

            let fl = EFlags::from_elf_section_flags(s);

            let start_frame = Frame::caddr(s.start_addr());
            let end_frame   = Frame::caddr(s.end_addr() - 1);
            for fr in Frame::range_inclusive(start_frame, end_frame) {
                map.idmap(&fr, fl, a);
            }
        }

        let vga_buf = &Frame::caddr(0xB8000);
        map.idmap(vga_buf, WRITABLE, a);

        let mb_start = Frame::caddr(b.start_addr());
        let mb_end   = Frame::caddr(b.end_addr() - 1);

        Frame::range_inclusive(mb_start, mb_end).for_each(|fr| map.idmap(&fr, PRESENT, a));
    });

    let old_t = at.switch(&new_t);

    let old_p4_p = Page::caddr(old_t.p4_frame.addr_ptr());

    at.unmap(old_p4_p, a);

    println!("\nguard page at {:#x}", old_p4_p.start_addr());

    at
}

pub fn init(boot_info : &BootInfo) -> MemoryController {
    once!("mem::init cannot be called twice");

    let memory_map_tag = boot_info.memory_map_tag()
        .expect("Memory map tag required");
    
    println!("memory blocks:");

    memory_map_tag
        .memo()
        .for_each(|block| {
            println!("\tstart: {:#x}, len: {:#x}",
                     block.base, block.len);
        });
    
    let elf_sections_tag = boot_info.elf_sections_tag()
        .expect("Elf sections tag required");

    println!("\nelf sections:");
    
    elf_sections_tag
        .elf_sections()
        .for_each(|section| {
            println!("\taddr: 0x{:<10x} size: 0x{:<8x} flags: {:#x}",
                     section.start_addr(), section.end_addr(), section.fflg());
        });

    let kernel_start = elf_sections_tag
        .elf_sections()
        .filter(|s| s.is_allocated())
        .map(|s| s.start_addr())
        .min().unwrap();
    
    let kernel_end = elf_sections_tag
        .elf_sections()
        .filter(|s| s.is_allocated())
        .map(|s| s.start_addr() + s.size())
        .max().unwrap();
    
    let mut frame_allocator = AreaAllocator::new(
        kernel_start as usize, kernel_end as usize, 
        boot_info.start_addr(), boot_info.end_addr(), 
        memory_map_tag.memo());

    let mut active_table = kernel_remap(&mut frame_allocator, boot_info);

    let heap_sp = Page::caddr(HEAP_START);
    let heap_ep = Page::caddr(HEAP_START + HEAP_SIZE - 1);

    Page::range_inclusive(heap_sp, heap_ep).for_each(|p| active_table.map(p, WRITABLE, &mut frame_allocator));
    
    let stack_sp = heap_ep + 1;
    let stack_ep = stack_sp + STACK_ALLOCATOR_SIZE;
    let stack_allocator = stack::StackAllocator::new(Page::range_inclusive(stack_sp, stack_ep));

    println!("\nkernel\t\t at: 0x{:<8x} - {:<8x}", kernel_start, kernel_end);
    println!("multiboot at: 0x{:<8x} - 0x{:<8x}", boot_info.start_addr(), boot_info.end_addr());
    println!("heap \t at: 0x{:<8x} - 0x{:<8x}", HEAP_START, HEAP_START + HEAP_SIZE - 1);
    println!("stack \t\t at: 0x{:<8x} - 0x{:<8x}\n\n", stack_sp.i, stack_ep.i);

    MemoryController {
        _at   : active_table,
        _fr_a : frame_allocator,
        _st_a : stack_allocator,
    }
}
