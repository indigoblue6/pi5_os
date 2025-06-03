/* Raspberry Pi 5 Linker Script */
/* Based on Ubuntu Pi5 kernel memory layout */

ENTRY(_start)

MEMORY
{
    RAM : ORIGIN = 0x200000, LENGTH = 0x10000000
}

SECTIONS
{
    . = 0x200000;
    
    .text.boot : {
        *(.text.boot)
    } > RAM
    
    .text : {
        . = ALIGN(4096);
        *(.text*)
    } > RAM
    
    .rodata : {
        . = ALIGN(4096);
        *(.rodata*)
    } > RAM
    
    .data : {
        . = ALIGN(4096);
        *(.data*)
    } > RAM
    
    .bss : {
        . = ALIGN(8);
        _BSS_START = .;
        *(.bss*)
        *(COMMON)
        . = ALIGN(8);
        _BSS_END = .;
    } > RAM
    
    /* Stack grows downward from here */
    . = ALIGN(4096);
    . += 0x200000;  /* 2MB stack */
    STACK_POINTER = .;
    
    /DISCARD/ : {
        *(.comment)
        *(.gnu*)
        *(.note*)
        *(.eh_frame*)
    }
}
