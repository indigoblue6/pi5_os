// Raspberry Pi 5 Interrupt Controller (GIC-400)
// Based on ARM Generic Interrupt Controller v2.0 specification

use crate::uart::Uart;

// GIC-400 Base addresses for Pi5
const GIC_DISTRIBUTOR_BASE: u64 = 0x2000_1000;
const GIC_CPU_INTERFACE_BASE: u64 = 0x2000_2000;

// Distributor registers
const GICD_CTLR: u32 = 0x000;      // Distributor Control Register
const GICD_TYPER: u32 = 0x004;     // Interrupt Controller Type Register
const GICD_ISENABLER: u32 = 0x100; // Interrupt Set-Enable Registers
const GICD_ICENABLER: u32 = 0x180; // Interrupt Clear-Enable Registers
const GICD_ISPENDR: u32 = 0x200;   // Interrupt Set-Pending Registers
const GICD_ICPENDR: u32 = 0x280;   // Interrupt Clear-Pending Registers
const GICD_IPRIORITYR: u32 = 0x400; // Interrupt Priority Registers
const GICD_ITARGETSR: u32 = 0x800; // Interrupt Processor Targets Registers

// CPU Interface registers
const GICC_CTLR: u32 = 0x000;      // CPU Interface Control Register
const GICC_PMR: u32 = 0x004;       // Interrupt Priority Mask Register
const GICC_IAR: u32 = 0x00C;       // Interrupt Acknowledge Register
const GICC_EOIR: u32 = 0x010;      // End of Interrupt Register

// Interrupt numbers for Pi5
const IRQ_TIMER: u32 = 64;          // System Timer
const IRQ_UART0: u32 = 153;         // UART0 (RP1)
const IRQ_GPIO: u32 = 113;          // GPIO controller

pub struct InterruptController {
    gic_dist_base: u64,
    gic_cpu_base: u64,
    uart: &'static mut Uart,
}

impl InterruptController {
    pub fn new(uart: &'static mut Uart) -> Self {
        Self {
            gic_dist_base: GIC_DISTRIBUTOR_BASE,
            gic_cpu_base: GIC_CPU_INTERFACE_BASE,
            uart,
        }
    }

    pub fn init(&mut self) -> Result<(), &'static str> {
        self.uart.write_str("Initializing GIC-400 interrupt controller...\r\n");

        // Initialize Distributor
        self.init_distributor()?;
        
        // Initialize CPU Interface
        self.init_cpu_interface()?;

        // Enable specific interrupts
        self.enable_interrupt(IRQ_TIMER);
        self.enable_interrupt(IRQ_UART0);
        self.enable_interrupt(IRQ_GPIO);

        self.uart.write_str("GIC-400 initialized successfully\r\n");
        Ok(())
    }

    fn init_distributor(&mut self) -> Result<(), &'static str> {
        // Disable distributor
        self.write_distributor_reg(GICD_CTLR, 0);

        // Read number of interrupt lines
        let typer = self.read_distributor_reg(GICD_TYPER);
        let num_lines = ((typer & 0x1F) + 1) * 32;
        
        // Disable all interrupts
        for i in (0..num_lines).step_by(32) {
            self.write_distributor_reg(GICD_ICENABLER + (i / 8), 0xFFFFFFFF);
        }

        // Clear all pending interrupts
        for i in (0..num_lines).step_by(32) {
            self.write_distributor_reg(GICD_ICPENDR + (i / 8), 0xFFFFFFFF);
        }

        // Set priority for all interrupts (lower number = higher priority)
        for i in (0..num_lines).step_by(4) {
            self.write_distributor_reg(GICD_IPRIORITYR + i, 0xA0A0A0A0);
        }

        // Set target processor to CPU0 for all interrupts
        for i in (32..num_lines).step_by(4) {
            self.write_distributor_reg(GICD_ITARGETSR + i, 0x01010101);
        }

        // Enable distributor
        self.write_distributor_reg(GICD_CTLR, 1);

        Ok(())
    }

    fn init_cpu_interface(&mut self) -> Result<(), &'static str> {
        // Set priority mask to allow all interrupts
        self.write_cpu_reg(GICC_PMR, 0xFF);

        // Enable CPU interface
        self.write_cpu_reg(GICC_CTLR, 1);

        Ok(())
    }

    pub fn enable_interrupt(&mut self, irq: u32) {
        let reg_offset = (irq / 32) * 4;
        let bit_offset = irq % 32;
        let reg_addr = GICD_ISENABLER + reg_offset;
        
        let current = self.read_distributor_reg(reg_addr);
        self.write_distributor_reg(reg_addr, current | (1 << bit_offset));
    }

    pub fn disable_interrupt(&mut self, irq: u32) {
        let reg_offset = (irq / 32) * 4;
        let bit_offset = irq % 32;
        let reg_addr = GICD_ICENABLER + reg_offset;
        
        self.write_distributor_reg(reg_addr, 1 << bit_offset);
    }

    pub fn handle_interrupt(&mut self) -> Option<u32> {
        // Read interrupt acknowledge register
        let iar = self.read_cpu_reg(GICC_IAR);
        let irq = iar & 0x3FF;
        
        // Check if it's a spurious interrupt
        if irq >= 1020 {
            return None;
        }

        // Handle specific interrupts
        match irq {
            IRQ_TIMER => {
                self.uart.write_str("Timer interrupt received\r\n");
            }
            IRQ_UART0 => {
                self.uart.write_str("UART interrupt received\r\n");
            }
            IRQ_GPIO => {
                self.uart.write_str("GPIO interrupt received\r\n");
            }
            _ => {
                self.uart.write_str("Unknown interrupt: ");
                self.uart.put_hex(irq);
                self.uart.write_str("\r\n");
            }
        }

        // End of interrupt
        self.write_cpu_reg(GICC_EOIR, iar);

        Some(irq)
    }

    fn read_distributor_reg(&self, offset: u32) -> u32 {
        unsafe {
            core::ptr::read_volatile((self.gic_dist_base + offset as u64) as *const u32)
        }
    }

    fn write_distributor_reg(&mut self, offset: u32, value: u32) {
        unsafe {
            core::ptr::write_volatile((self.gic_dist_base + offset as u64) as *mut u32, value);
        }
    }

    fn read_cpu_reg(&self, offset: u32) -> u32 {
        unsafe {
            core::ptr::read_volatile((self.gic_cpu_base + offset as u64) as *const u32)
        }
    }

    fn write_cpu_reg(&mut self, offset: u32, value: u32) {
        unsafe {
            core::ptr::write_volatile((self.gic_cpu_base + offset as u64) as *mut u32, value);
        }
    }
}

// Exception vector table setup
core::arch::global_asm!(
    "
    .section .text._start_vectors
    .balign 0x800
    .global _start_vectors
_start_vectors:
    // Current EL with SP0
    .balign 0x80
    b   sync_exception_sp0
    .balign 0x80
    b   irq_exception_sp0
    .balign 0x80
    b   fiq_exception_sp0
    .balign 0x80
    b   serror_exception_sp0

    // Current EL with SPx
    .balign 0x80
    b   sync_exception_spx
    .balign 0x80
    b   irq_exception_spx
    .balign 0x80
    b   fiq_exception_spx
    .balign 0x80
    b   serror_exception_spx

    // Lower EL using AArch64
    .balign 0x80
    b   sync_exception_aarch64
    .balign 0x80
    b   irq_exception_aarch64
    .balign 0x80
    b   fiq_exception_aarch64
    .balign 0x80
    b   serror_exception_aarch64

    // Lower EL using AArch32
    .balign 0x80
    b   sync_exception_aarch32
    .balign 0x80
    b   irq_exception_aarch32
    .balign 0x80
    b   fiq_exception_aarch32
    .balign 0x80
    b   serror_exception_aarch32

sync_exception_sp0:
    b   sync_exception_sp0

irq_exception_sp0:
    b   irq_handler

fiq_exception_sp0:
    b   fiq_exception_sp0

serror_exception_sp0:
    b   serror_exception_sp0

sync_exception_spx:
    b   sync_exception_spx

irq_exception_spx:
    b   irq_handler

fiq_exception_spx:
    b   fiq_exception_spx

serror_exception_spx:
    b   serror_exception_spx

sync_exception_aarch64:
    b   sync_exception_aarch64

irq_exception_aarch64:
    b   irq_handler

fiq_exception_aarch64:
    b   fiq_exception_aarch64

serror_exception_aarch64:
    b   serror_exception_aarch64

sync_exception_aarch32:
    b   sync_exception_aarch32

irq_exception_aarch32:
    b   irq_handler

fiq_exception_aarch32:
    b   fiq_exception_aarch32

serror_exception_aarch32:
    b   serror_exception_aarch32

irq_handler:
    // Save registers
    stp x29, x30, [sp, #-16]!
    stp x27, x28, [sp, #-16]!
    stp x25, x26, [sp, #-16]!
    stp x23, x24, [sp, #-16]!
    stp x21, x22, [sp, #-16]!
    stp x19, x20, [sp, #-16]!
    stp x17, x18, [sp, #-16]!
    stp x15, x16, [sp, #-16]!
    stp x13, x14, [sp, #-16]!
    stp x11, x12, [sp, #-16]!
    stp x9, x10, [sp, #-16]!
    stp x7, x8, [sp, #-16]!
    stp x5, x6, [sp, #-16]!
    stp x3, x4, [sp, #-16]!
    stp x1, x2, [sp, #-16]!
    str x0, [sp, #-16]!

    // Call Rust interrupt handler
    bl  rust_irq_handler

    // Restore registers
    ldr x0, [sp], #16
    ldp x1, x2, [sp], #16
    ldp x3, x4, [sp], #16
    ldp x5, x6, [sp], #16
    ldp x7, x8, [sp], #16
    ldp x9, x10, [sp], #16
    ldp x11, x12, [sp], #16
    ldp x13, x14, [sp], #16
    ldp x15, x16, [sp], #16
    ldp x17, x18, [sp], #16
    ldp x19, x20, [sp], #16
    ldp x21, x22, [sp], #16
    ldp x23, x24, [sp], #16
    ldp x25, x26, [sp], #16
    ldp x27, x28, [sp], #16
    ldp x29, x30, [sp], #16

    eret
    "
);

// Interrupt controller instance
static mut INTERRUPT_CONTROLLER: Option<InterruptController> = None;

#[no_mangle]
extern "C" fn rust_irq_handler() {
    unsafe {
        if let Some(ref mut ic) = INTERRUPT_CONTROLLER {
            ic.handle_interrupt();
        }
    }
}

pub fn init_interrupts(uart: &'static mut Uart) -> Result<(), &'static str> {
    unsafe {
        INTERRUPT_CONTROLLER = Some(InterruptController::new(uart));
        if let Some(ref mut ic) = INTERRUPT_CONTROLLER {
            ic.init()?;
        }
    }

    // Install vector table
    unsafe {
        extern "C" {
            static _start_vectors: u8;
        }
        let vbar = &_start_vectors as *const u8 as u64;
        core::arch::asm!(
            "msr vbar_el1, {}",
            in(reg) vbar
        );
    }

    // Enable interrupts
    unsafe {
        core::arch::asm!(
            "msr daifclr, #2"  // Clear IRQ mask
        );
    }

    Ok(())
}
