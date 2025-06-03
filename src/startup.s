.section ".text._start"
.global _start

_start:
    // Ensure we're running on core 0
    mrs x0, mpidr_el1
    and x0, x0, #3
    cbnz x0, halt
    
    // Set stack pointer - Pi5 has more memory
    ldr x0, =0x80000000
    mov sp, x0
    
    // Clear BSS section
    ldr x0, =_BSS_START
    ldr x1, =_BSS_END
    
clear_bss:
    cmp x0, x1
    b.ge clear_done
    str xzr, [x0], #8
    b clear_bss
    
clear_done:
    // Jump to Rust main function
    bl rust_main
    
halt:
    wfe
    b halt