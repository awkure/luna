// -*- mode: rust; -*-

//! # x86 Interrupt handler functions

use x86_64::structures::idt::{ExceptionStackFrame, PageFaultErrorCode};

pub extern "x86-interrupt" fn __breakpoint_handler(stack_frame : &mut ExceptionStackFrame) {
    println!("\nEXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn __page_fault_handler( stack_frame : &mut ExceptionStackFrame
                                                  , error_code  : PageFaultErrorCode ) {
    use x86_64::registers::control_regs;
    println!("\nEXCEPTION: PAGE FAULT [{:#x}] ; e: [{:?}]\n{:#?}", control_regs::cr2(), error_code, stack_frame);
    loop {}
}

pub extern "x86-interrupt" fn __double_fault_handler(stack_frame : &mut ExceptionStackFrame, ec : u64) {
    println!("\nEXCEPTION: DOUBLE FAULT [e = {}]\n{:#?}", ec, stack_frame);
    loop {}
}

pub extern "x86-interrupt" fn __invalid_opcode_handler(stack_frame : &mut ExceptionStackFrame) {
    println!("\nEXCEPTION: INVALID OPCODE [{:#x}]\n{:#?}", stack_frame.instruction_pointer, stack_frame);
    loop {}
}

pub extern "x86-interrupt" fn __divide_by_zero_handler(stack_frame : &mut ExceptionStackFrame) {
    println!("\nEXCEPTION: DIVIDE BY ZERO\n{:#?}", stack_frame);
    loop {}
}
