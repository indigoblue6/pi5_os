// Entry point and kernel initialization - Pi5Hack style implementation

#![no_std]
#![no_main]

global_asm!(include_str!("startup.s"));

mod uart;
mod mmu;
mod process;
mod timer;
mod shell;
mod interrupt;
mod gpio;
mod filesystem;
mod syscalls;
mod signals;
mod ipc;
mod users;
mod unix_commands;

use core::{
    arch::global_asm,
    panic::PanicInfo,
    ptr,
};

use uart::UART;
use syscalls::init_syscalls;
use signals::SignalHandler;
use ipc::IPCManager;
use users::UserManager;

// Panic handler - pi5_hack style
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // Try to print the panic info if possible
    UART.write_str("\n\n*** KERNEL PANIC ***\n");
    if let Some(location) = info.location() {
        UART.write_str("Location: ");
        UART.write_str(location.file());
        UART.write_str(":");
        // Convert line number to string manually since to_string() is not available
        UART.put_hex(location.line());
        UART.write_str("\n");
    }
    
    // Try to print the panic message
    UART.write_str("Message: ");
    // Use the payload directly since message() doesn't return Option
    use core::fmt::Write;
    let mut uart = UART;  // Now works because Uart implements Copy
    let _ = write!(&mut uart, "{}", info.message());
    UART.write_str("\n");
    
    // Data synchronization barrier
    unsafe {
        core::arch::asm!("dsb sy");
    }
    
    // Halt CPU in low-power mode
    loop {
        unsafe {
            core::arch::asm!("wfe");
        }
    }
}

// BSS section symbols from linker
extern "C" {
    static mut _BSS_START: u64;
    static mut _BSS_END: u64;
}

// Clear BSS section - pi5_hack style
fn clear_bss() {
    unsafe {
        let bss_start = &mut _BSS_START as *mut u64;
        let bss_end = &mut _BSS_END as *mut u64;
        let bss_size = bss_end.offset_from(bss_start) as usize;
        
        ptr::write_bytes(bss_start as *mut u8, 0, bss_size * 8);
    }
}

// Initialize UNIX subsystems
fn init_unix_subsystems() {
    UART.write_str("Initializing UNIX subsystems...\r\n");
    
    // Initialize syscall manager
    UART.write_str("  - System calls: ");
    init_syscalls();
    UART.write_str("OK\r\n");
    
    // Initialize signal manager
    UART.write_str("  - Signal handling: ");
    let signal_handler = SignalHandler::new();
    UART.write_str("OK\r\n");
    
    // Initialize IPC manager
    UART.write_str("  - Inter-process communication: ");
    let ipc_manager = IPCManager::new();
    UART.write_str("OK\r\n");
    
    // Initialize user manager with root user
    UART.write_str("  - User management: ");
    let mut user_manager = UserManager::new();
    UART.write_str("OK\r\n");
    
    UART.write_str("UNIX subsystems initialized!\r\n\r\n");
}

// Basic memory test
fn test_memory_operations() {
    // Stack allocation test
    let mut test_array = [0u8; 256];
    for i in 0..256 {
        test_array[i] = (i % 256) as u8;
    }
    
    // Simple checksum
    let mut checksum = 0u32;
    for byte in &test_array {
        checksum = checksum.wrapping_add(*byte as u32);
    }
    
    UART.write_str("  Checksum: ");
    UART.put_hex(checksum);
    UART.write_str("\n");
}

// Basic timer test (software delay)
fn test_timer_functions() {
    UART.write_str("  Delay test: ");
    
    for i in 1..=5 {
        // Software delay
        for _ in 0..1000000 {
            unsafe { core::arch::asm!("nop"); }
        }
        UART.write_char((b'0' + i) as char);
        UART.write_char(' ');
    }
    UART.write_str("done\n");
}

// Basic GPIO test (simplified)
fn test_gpio_functions() {
    UART.write_str("  GPIO basic test: ");
    
    // Pi5のGPIO基本アドレス（BCM2712）
    const GPIO_BASE: u64 = 0x107d200000;
    
    unsafe {
        // 簡単なGPIOアクセステスト（読み取りのみ）
        let gpio_status = core::ptr::read_volatile(GPIO_BASE as *const u32);
        UART.write_str("status=");
        UART.put_hex(gpio_status);
        UART.write_str(" ");
    }
    
    UART.write_str("ok\n");
}

// Pi5Hack OS - main entry point (exact style from pi5_hack)
#[no_mangle]
pub extern "C" fn rust_main() -> ! {
    // Clear BSS first
    clear_bss();

    // Initialize UART using pi5_hack method
    unsafe {
        if let Err(_) = UART.init() {
            loop { core::arch::asm!("wfe"); }
        }
    }
    
    // Comprehensive test loop with all hardware tests
    const MAX_TEST_CYCLES: u32 = 1; // Run 3 test cycles before starting shell
    let mut counter = 0u32;
    
    while counter < MAX_TEST_CYCLES {
        UART.write_str("=== PI5HACK-BOOT CYCLE ");
        UART.put_hex(counter);
        UART.write_str(" ===\r\n");
        
        // Test 1: Memory operations
        UART.write_str("1. Memory Test:\r\n");
        test_memory_operations();
        
        // Test 2: Timer functions (delay test)
        UART.write_str("2. Timer Test:\r\n");
        test_timer_functions();
        
        // Test 3: GPIO basic operations
        UART.write_str("3. GPIO Test:\r\n");
        test_gpio_functions();
        
        // Test 4: UART functionality test
        UART.write_str("4. UART Test:\r\n");
        UART.write_str("  Character output: ");
        for ch in "Hello Pi5!".chars() {
            UART.write_char(ch);
        }
        UART.write_str("\r\n");
        
        // Test 5: Stack and heap test
        UART.write_str("5. Stack Test:\r\n");
        UART.write_str("  Local vars: ");
        let local_var1 = 0xDEADBEEF;
        let local_var2 = 0xCAFEBABE;
        UART.put_hex(local_var1);
        UART.write_str(" ");
        UART.put_hex(local_var2);
        UART.write_str("\r\n");
        
        // Summary
        UART.write_str("=== CYCLE ");
        UART.put_hex(counter);
        UART.write_str(" COMPLETE ===\r\n\r\n");
        
        counter = counter.wrapping_add(1);
        
        if counter < MAX_TEST_CYCLES {
            // Delay between test cycles
            UART.write_str("Waiting 3 seconds for next cycle...\r\n");
            for _ in 0..144_000_000 { // ~3 seconds at 48MHz
                unsafe { core::arch::asm!("nop"); }
            }
        }
    }
    
    // Hardware tests completed, initialize UNIX subsystems
    init_unix_subsystems();
    
    // Start interactive shell
    UART.write_str("\r\n");
    UART.write_str("========================================\r\n");
    UART.write_str("   UNIX-COMPATIBLE OS READY!           \r\n");
    UART.write_str("   Starting Interactive Shell...       \r\n");
    UART.write_str("========================================\r\n");
    UART.write_str("\r\n");
    
    // Start the interactive shell
    let mut shell = shell::Shell::new();
    shell.run();
    
    // Shell has exited (e.g., user typed 'exit'), show shutdown message
    UART.write_str("\r\n");
    UART.write_str("========================================\r\n");
    UART.write_str("   SHELL EXITED - SYSTEM SHUTDOWN       \r\n");
    UART.write_str("========================================\r\n");
    
    // Infinite loop since rust_main is declared as -> ! (never returns)
    loop {
        unsafe { core::arch::asm!("wfe"); }
    }
}