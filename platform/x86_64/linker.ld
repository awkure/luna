ENTRY(_start)

SECTIONS {
    . = 0x1000000;

    .boot :
    {
        KEEP(*(.multiboot))
        . = ALIGN(4K);
    }

    .rodata : 
    {
        *(.rodata .rodata.*)
        . = ALIGN(4K);
    }

    .text :
    {
        *(.text .text.*)
        . = ALIGN(4K);
    }

    .bss :
    {
        *(.bss .bss.*)
        . = ALIGN(4K);
    }

    .data : 
    {
        *(.data .data.*)
        . = ALIGN(4K);
    }

    .got :
    {
        *(.got)
        . = ALIGN(4K);
    }

    .got.plt : 
    {
        *(.got.plt)
    }

    .data.rel.ro : ALIGN(4K) 
    {
        *(.data.rel.ro.local*) 
        *(data.rel.ro .data.rel.ro.*)
        . = ALIGN(4K);
    }

    .gcc_except_table : ALIGN(4K) 
    {
        *(.gcc_except_table)
        . = ALIGN(4K);
    }
}
