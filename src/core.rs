// -*- mode: rust; -*-

// Core module of the operating system.

#![feature( asm
          , alloc
          , allocator_api
          , global_allocator
          , abi_x86_interrupt
          , lang_items
          , allow_internal_unstable
          , const_fn
          , specialization
          , const_unique_new
          , const_atomic_usize_new
          , const_atomic_ptr_new
          , use_spin
          , use_extern_macros
          , use_nested_groups
          , unique )]

#![allow( unknown_lints
        , empty_loop
        , unreadable_literal
        , uncoditional_recursion
        , identity_op
        , unused_imports
        , dead_code
        , unused_macros )]

#![no_std]

extern crate rlibc;
#[macro_use] extern crate lazy_static;

extern crate spin;
extern crate x86_64;
extern crate bit_field;

#[macro_use] extern crate bitflags;
#[macro_use] extern crate alloc;

#[macro_use] mod kernel;

use kernel::{ 
    mem::{ self
         , alloc::heap::HeapAllocator}, 
    bits, 
    vga ,
};

#[global_allocator]
static HEAP_ALLOCATOR : HeapAllocator = HeapAllocator::blank();

#[lang = "panic_fmt"]
#[cfg(not(test))]
#[no_mangle]
pub extern fn panic_fmt(
    fmt  : core::fmt::Arguments, 
    file : &'static str, 
    line : u32
    ) -> ! 
{
    println!("\n\n*** {} at {}:", file, line);
    println!("\t\t {}", fmt);
    loop {}
}


#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn _Unwind_Resume() -> ! {
  panic!()
}


#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Clone,Copy)]
pub enum _Unwind_Reason_Code
{
      _URC_NO_REASON
    , _URC_FOREIGN_EXCEPTION_CAUGHT
    , _URC_FATAL_PHASE2_ERROR
    , _URC_FATAL_PHASE1_ERROR
    , _URC_NORMAL_STOP
    , _URC_END_OF_STACK
    , _URC_HANDLER_FOUND
    , _URC_INSTALL_CONTEXT
    , _URC_CONTINUE_UNWIND
}


#[allow(non_camel_case_types)]
#[derive(Clone,Copy)]
pub struct _Unwind_Context;


#[allow(non_camel_case_types)]
pub type _Unwind_Action = u32;
static _UA_SEARCH_PHASE : _Unwind_Action = 1;


#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Clone,Copy)]
pub struct _Unwind_Exception
{
	exception_class   : u64,
	exception_cleanup : fn(_Unwind_Reason_Code, *const _Unwind_Exception),
	private           : [u64 ; 2],
}


#[lang="eh_personality"]
#[no_mangle]
pub fn rust_eh_personality(
	_version : isize, 
    _actions : _Unwind_Action, 
    _exception_class  : u64,
	_exception_object : &_Unwind_Exception, 
    _context : &_Unwind_Context
	) -> _Unwind_Reason_Code
{
	loop{}
}


#[no_mangle]
pub extern "C" fn main(_mb_addr : usize) {
    use kernel::mem::globals;

    vga::clear_screen();

    bits::init();
    
    let memory_controller = &mut mem::init(unsafe { kernel::boot::load(_mb_addr) });

    unsafe {
        HEAP_ALLOCATOR.lock().init(globals::HEAP_START, globals::HEAP_START + globals::HEAP_SIZE);
    }

    kernel::interrupt::init(memory_controller);

    x86_64::instructions::interrupts::int3();

    loop {}
}
