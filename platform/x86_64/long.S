global _start_long_mode

section .text 
bits 64

_start_long_mode:
    xor ax, ax
    mov ss, ax
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax 

    extern main
    call   main

    hlt
