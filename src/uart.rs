// Pi5 UART Driver - Based on pi5_hack early_uart implementation
// Reference: pi5_hack/baremetal/XX_early_uart

use core::{ptr, fmt};
use core::fmt::Write;

// BCM2712 UART register addresses - EXACT from pi5_hack early_uart
const BCM2712_UART_BASE: u64 = 0x10_7d00_1000;
const BCM2712_UART_DR: *mut u32 = BCM2712_UART_BASE as *mut u32;
const BCM2712_UART_FLAG: *mut u32 = (BCM2712_UART_BASE + 0x18) as *mut u32;

// Flag Register bits - EXACT from pi5_hack early_uart
const UART_FR_RXFE: u32 = 1 << 4;  // RX FIFO empty
const UART_FR_TXFF: u32 = 1 << 5;  // TX FIFO full

// Pi5 UART struct - super simple version based on early_uart
#[derive(Copy, Clone)]
pub struct Uart;

// Simple implementation using EarlyUart pattern
impl Uart {
    pub const fn new() -> Self {
        Self
    }
    
    /// Pi5 UART initialization - very basic from early_uart
    pub unsafe fn init(&self) -> Result<(), &'static str> {
        // No initialization needed for early UART - already configured by bootloader
        Ok(())
    }
    
    // Write a character - direct from early_uart
    pub fn write_char(&self, c: char) {
        // Handle newline conversion to CRLF
        if c == '\n' {
            self.write_char_raw('\r');
        }
        
        self.write_char_raw(c);
    }
    
    // Write a raw character without CRLF conversion
    fn write_char_raw(&self, c: char) {
        unsafe {
            // Wait for TX FIFO not full
            while ptr::read_volatile(BCM2712_UART_FLAG) & UART_FR_TXFF != 0 {
                core::arch::asm!("nop");
            }
            
            // Write character
            ptr::write_volatile(BCM2712_UART_DR, c as u32);
        }
    }
    
    // Write a string
    pub fn write_str(&self, s: &str) {
        for c in s.chars() {
            self.write_char(c);
        }
    }
    
    // Read a character
    pub fn read_char(&self) -> Option<char> {
        unsafe {
            // Check if RX FIFO is empty
            if ptr::read_volatile(BCM2712_UART_FLAG) & UART_FR_RXFE != 0 {
                return None; // No data available
            }
            
            // Read character
            let data = ptr::read_volatile(BCM2712_UART_DR);
            Some((data & 0xFF) as u8 as char)
        }
    }
    
    // Read a character with timeout
    pub fn read_char_timeout(&self, timeout: u32) -> Option<char> {
        for _ in 0..timeout {
            if let Some(c) = self.read_char() {
                return Some(c);
            }
            unsafe { core::arch::asm!("nop"); }
        }
        None
    }
    
    // Write byte array
    pub fn write(&self, data: &[u8]) -> usize {
        for (i, &byte) in data.iter().enumerate() {
            unsafe {
                // Check if TX FIFO is full
                if ptr::read_volatile(BCM2712_UART_FLAG) & UART_FR_TXFF != 0 {
                    return i; // Return how many bytes were written
                }
                ptr::write_volatile(BCM2712_UART_DR, byte as u32);
            }
        }
        data.len()
    }

    
    // Read byte array
    pub fn read(&self, data: &mut [u8]) -> usize {
        for (i, byte) in data.iter_mut().enumerate() {
            unsafe {
                // Check if RX FIFO is empty
                if ptr::read_volatile(BCM2712_UART_FLAG) & UART_FR_RXFE != 0 {
                    return i; // Return how many bytes were read
                }
                *byte = ptr::read_volatile(BCM2712_UART_DR) as u8;
            }
        }
        data.len()
    }
    
    /// Hex output for debugging
    pub fn put_hex(&self, num: u32) {
        let hex_chars = b"0123456789ABCDEF";
        self.write_str("0x");
        for i in (0..8).rev() {
            let nibble = (num >> (i * 4)) & 0xF;
            self.write_char(hex_chars[nibble as usize] as char);
        }
    }
}

// Implement Write trait for formatting output (print!, println! macros)
impl Write for Uart {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        // Call the original write_str method to avoid recursion
        Uart::write_str(self, s);
        Ok(())
    }
}

// Global UART instance - pi5_hack style
pub static UART: Uart = Uart::new();

// Convenience macros for printing
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::uart::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

// Internal printer function used by macros
#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    let mut uart = UART;  // Now works because Uart implements Copy
    uart.write_fmt(args).unwrap();
}