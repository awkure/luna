// -*- mode: rust; -*-

use spin::Once;

use x86_64::{
    VirtualAddress,
    structures::{ idt::Idt
                , gdt::SegmentSelector
                , tss::TaskStateSegment },
    instructions::{ segmentation::set_cs
                  , tables::load_tss },
};

use super::{
    gdt,
    handlers::*,
};

use kernel::mem::control::MemoryController;

pub const DOUBLE_FAULT_IST_IDX : usize = 0;

static TSS : Once<TaskStateSegment> = Once::new();
static GDT : Once<gdt::GDT> = Once::new();

/// Enable hardware interrupts
pub unsafe fn enable() {
    asm!("sti");
}

/// Disable hardware interrupts 
pub unsafe fn disable() {
    asm!("cli");
}

lazy_static! {
    static ref IDT : Idt = {
        let mut idt = Idt::new();

        idt.breakpoint.set_handler_fn(__breakpoint_handler);
        idt.divide_by_zero.set_handler_fn(__divide_by_zero_handler);
        idt.invalid_opcode.set_handler_fn(__invalid_opcode_handler);
        idt.page_fault.set_handler_fn(__page_fault_handler);

        unsafe {
            idt.double_fault.set_handler_fn(__double_fault_handler)
                            .set_stack_index(DOUBLE_FAULT_IST_IDX as u16);
        }

        idt
    };
}

pub fn init(mc : &mut MemoryController) {
    let double_fault_stack = mc.alloc(1).expect("double fault stack cannot be allocated");

    let tss = TSS.call_once(|| {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_IDX] = VirtualAddress(double_fault_stack.top());
        tss
    });

    let mut code_selector = SegmentSelector(0);
    let mut tss_selector  = SegmentSelector(0);

    let gdt = GDT.call_once(|| {
        let mut gdt = gdt::GDT::new();
        
        code_selector = gdt.add_entry(&gdt::Descriptor::kernel_code_segment());
        tss_selector  = gdt.add_entry(&gdt::Descriptor::tss_segment(tss));

        gdt
    });

    gdt.load();

    unsafe {
        set_cs(code_selector);
        load_tss(tss_selector);
    }

    IDT.load();
}
