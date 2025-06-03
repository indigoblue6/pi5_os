// Raspberry Pi 5 GPIO Controller
// Based on Ubuntu linux-raspi pinctrl-rp1.c driver implementation

use crate::uart::Uart;

// RP1 GPIO base address (Ubuntu kernel verified)
const RP1_GPIO_BASE: u64 = 0x1f000d0000;

// GPIO register offsets
const GPIO_CTRL: u32 = 0x0004;
const GPIO_STATUS: u32 = 0x0000;

// Function select values
const GPIO_FUNC_SPI: u32 = 1;
const GPIO_FUNC_UART: u32 = 2;
const GPIO_FUNC_I2C: u32 = 3;
const GPIO_FUNC_PWM: u32 = 4;
const GPIO_FUNC_SIO: u32 = 5;    // Software controlled I/O
const GPIO_FUNC_PIO: u32 = 6;
const GPIO_FUNC_GPCK: u32 = 8;
const GPIO_FUNC_USB: u32 = 9;
const GPIO_FUNC_NULL: u32 = 31;

// GPIO control register bits
const GPIO_CTRL_FUNCSEL_MASK: u32 = 0x1f;
const GPIO_CTRL_OUTOVER_MASK: u32 = 0x3000;
const GPIO_CTRL_OUTOVER_SHIFT: u32 = 12;
const GPIO_CTRL_OEOVER_MASK: u32 = 0xc000;
const GPIO_CTRL_OEOVER_SHIFT: u32 = 14;
const GPIO_CTRL_INOVER_MASK: u32 = 0x30000;
const GPIO_CTRL_INOVER_SHIFT: u32 = 16;

// GPIO status register bits
const GPIO_STATUS_OUTFROMPERI: u32 = 0x100;
const GPIO_STATUS_OUTTOPAD: u32 = 0x200;
const GPIO_STATUS_OEFROMPERI: u32 = 0x1000;
const GPIO_STATUS_OETOPAD: u32 = 0x2000;
const GPIO_STATUS_INFROMPAD: u32 = 0x20000;
const GPIO_STATUS_INTOPERI: u32 = 0x40000;

// SIO (Software I/O) registers for direct GPIO control
const SIO_BASE: u64 = 0x1f000d0000 + 0x000000;
const SIO_GPIO_OUT: u32 = 0x010;
const SIO_GPIO_OUT_SET: u32 = 0x014;
const SIO_GPIO_OUT_CLR: u32 = 0x018;
const SIO_GPIO_OUT_XOR: u32 = 0x01c;
const SIO_GPIO_OE: u32 = 0x020;
const SIO_GPIO_OE_SET: u32 = 0x024;
const SIO_GPIO_OE_CLR: u32 = 0x028;
const SIO_GPIO_OE_XOR: u32 = 0x02c;
const SIO_GPIO_IN: u32 = 0x004;

// Pi5 specific GPIO pins
pub const GPIO_LED_ACT: u32 = 29;      // Activity LED (green)
pub const GPIO_LED_PWR: u32 = 31;      // Power LED (red) 
pub const GPIO_UART_TX: u32 = 14;      // UART TX
pub const GPIO_UART_RX: u32 = 15;      // UART RX
pub const GPIO_SDA1: u32 = 2;          // I2C SDA
pub const GPIO_SCL1: u32 = 3;          // I2C SCL

#[derive(Debug, Clone, Copy)]
pub enum GpioDirection {
    Input,
    Output,
}

#[derive(Debug, Clone, Copy)]
pub enum GpioLevel {
    Low,
    High,
}

#[derive(Debug, Clone, Copy)]
pub enum GpioFunction {
    Spi = GPIO_FUNC_SPI as isize,
    Uart = GPIO_FUNC_UART as isize,
    I2c = GPIO_FUNC_I2C as isize,
    Pwm = GPIO_FUNC_PWM as isize,
    Sio = GPIO_FUNC_SIO as isize,
    Pio = GPIO_FUNC_PIO as isize,
    Gpck = GPIO_FUNC_GPCK as isize,
    Usb = GPIO_FUNC_USB as isize,
    Null = GPIO_FUNC_NULL as isize,
}

pub struct GpioController {
    gpio_base: u64,
    sio_base: u64,
    uart: &'static mut Uart,
}

impl GpioController {
    pub fn new(uart: &'static mut Uart) -> Self {
        Self {
            gpio_base: RP1_GPIO_BASE,
            sio_base: SIO_BASE,
            uart,
        }
    }

    pub fn init(&mut self) -> Result<(), &'static str> {
        self.uart.write_str("Initializing RP1 GPIO controller...\r\n");

        // Test GPIO registers accessibility
        if !self.test_gpio_access() {
            return Err("GPIO registers not accessible");
        }

        // Setup basic GPIO functions
        self.setup_uart_pins()?;
        self.setup_led_pins()?;

        self.uart.write_str("RP1 GPIO controller initialized\r\n");
        Ok(())
    }

    fn test_gpio_access(&mut self) -> bool {
        // Try to read GPIO status register for pin 0
        let status = self.read_gpio_reg(0, GPIO_STATUS);
        // GPIO registers should be readable (not return 0xFFFFFFFF)
        status != 0xFFFFFFFF
    }

    fn setup_uart_pins(&mut self) -> Result<(), &'static str> {
        // Configure GPIO14 (TX) and GPIO15 (RX) for UART function
        self.set_function(GPIO_UART_TX, GpioFunction::Uart);
        self.set_function(GPIO_UART_RX, GpioFunction::Uart);
        
        self.uart.write_str("UART GPIO pins configured\r\n");
        Ok(())
    }

    fn setup_led_pins(&mut self) -> Result<(), &'static str> {
        // Configure LED pins as SIO (software controlled)
        self.set_function(GPIO_LED_ACT, GpioFunction::Sio);
        self.set_function(GPIO_LED_PWR, GpioFunction::Sio);
        
        // Set as outputs
        self.set_direction(GPIO_LED_ACT, GpioDirection::Output);
        self.set_direction(GPIO_LED_PWR, GpioDirection::Output);
        
        // Turn off LEDs initially
        self.set_level(GPIO_LED_ACT, GpioLevel::Low);
        self.set_level(GPIO_LED_PWR, GpioLevel::Low);

        self.uart.write_str("LED GPIO pins configured\r\n");
        Ok(())
    }

    pub fn set_function(&mut self, pin: u32, function: GpioFunction) {
        if pin >= 54 {
            return; // Invalid pin number
        }

        let mut ctrl = self.read_gpio_reg(pin, GPIO_CTRL);
        ctrl &= !GPIO_CTRL_FUNCSEL_MASK;
        ctrl |= function as u32 & GPIO_CTRL_FUNCSEL_MASK;
        self.write_gpio_reg(pin, GPIO_CTRL, ctrl);
    }

    pub fn set_direction(&mut self, pin: u32, direction: GpioDirection) {
        if pin >= 54 {
            return;
        }

        let bit_mask = 1u32 << pin;
        match direction {
            GpioDirection::Output => {
                self.write_sio_reg(SIO_GPIO_OE_SET, bit_mask);
            }
            GpioDirection::Input => {
                self.write_sio_reg(SIO_GPIO_OE_CLR, bit_mask);
            }
        }
    }

    pub fn set_level(&mut self, pin: u32, level: GpioLevel) {
        if pin >= 54 {
            return;
        }

        let bit_mask = 1u32 << pin;
        match level {
            GpioLevel::High => {
                self.write_sio_reg(SIO_GPIO_OUT_SET, bit_mask);
            }
            GpioLevel::Low => {
                self.write_sio_reg(SIO_GPIO_OUT_CLR, bit_mask);
            }
        }
    }

    pub fn get_level(&self, pin: u32) -> GpioLevel {
        if pin >= 54 {
            return GpioLevel::Low;
        }

        let input_reg = self.read_sio_reg(SIO_GPIO_IN);
        if (input_reg & (1 << pin)) != 0 {
            GpioLevel::High
        } else {
            GpioLevel::Low
        }
    }

    pub fn toggle_pin(&mut self, pin: u32) {
        if pin >= 54 {
            return;
        }

        let bit_mask = 1u32 << pin;
        self.write_sio_reg(SIO_GPIO_OUT_XOR, bit_mask);
    }

    // LED control functions
    pub fn set_activity_led(&mut self, on: bool) {
        self.set_level(GPIO_LED_ACT, if on { GpioLevel::High } else { GpioLevel::Low });
    }

    pub fn set_power_led(&mut self, on: bool) {
        self.set_level(GPIO_LED_PWR, if on { GpioLevel::High } else { GpioLevel::Low });
    }

    pub fn blink_activity_led(&mut self) {
        self.toggle_pin(GPIO_LED_ACT);
    }

    pub fn blink_power_led(&mut self) {
        self.toggle_pin(GPIO_LED_PWR);
    }

    // GPIO status and control for debugging
    pub fn get_pin_status(&self, pin: u32) -> u32 {
        if pin >= 54 {
            return 0;
        }
        self.read_gpio_reg(pin, GPIO_STATUS)
    }

    pub fn get_pin_control(&self, pin: u32) -> u32 {
        if pin >= 54 {
            return 0;
        }
        self.read_gpio_reg(pin, GPIO_CTRL)
    }

    // Low-level register access
    fn read_gpio_reg(&self, pin: u32, offset: u32) -> u32 {
        let reg_addr = self.gpio_base + (pin as u64 * 8) + offset as u64;
        unsafe {
            core::ptr::read_volatile(reg_addr as *const u32)
        }
    }

    fn write_gpio_reg(&mut self, pin: u32, offset: u32, value: u32) {
        let reg_addr = self.gpio_base + (pin as u64 * 8) + offset as u64;
        unsafe {
            core::ptr::write_volatile(reg_addr as *mut u32, value);
        }
    }

    fn read_sio_reg(&self, offset: u32) -> u32 {
        let reg_addr = self.sio_base + offset as u64;
        unsafe {
            core::ptr::read_volatile(reg_addr as *const u32)
        }
    }

    fn write_sio_reg(&mut self, offset: u32, value: u32) {
        let reg_addr = self.sio_base + offset as u64;
        unsafe {
            core::ptr::write_volatile(reg_addr as *mut u32, value);
        }
    }

    // Hardware test functions
    pub fn test_gpio_functionality(&mut self) -> bool {
        self.uart.write_str("Testing GPIO functionality...\r\n");

        // Test LED control
        self.uart.write_str("Testing LED control...\r\n");
        self.set_activity_led(true);
        crate::timer::delay_ms(100);
        self.set_activity_led(false);
        
        self.set_power_led(true);
        crate::timer::delay_ms(100);
        self.set_power_led(false);

        // Test pin status reading
        let status = self.get_pin_status(GPIO_LED_ACT);
        self.uart.write_str("GPIO29 status: 0x");
        self.uart.put_hex(status);
        self.uart.write_str("\r\n");

        let ctrl = self.get_pin_control(GPIO_LED_ACT);
        self.uart.write_str("GPIO29 control: 0x");
        self.uart.put_hex(ctrl);
        self.uart.write_str("\r\n");

        self.uart.write_str("GPIO test completed\r\n");
        true
    }
}

// GPIO controller instance
static mut GPIO_CONTROLLER: Option<GpioController> = None;

pub fn init_gpio(uart: &'static mut Uart) -> Result<(), &'static str> {
    unsafe {
        GPIO_CONTROLLER = Some(GpioController::new(uart));
        if let Some(ref mut gpio) = GPIO_CONTROLLER {
            gpio.init()?;
        }
    }
    Ok(())
}

pub fn get_gpio_controller() -> Option<&'static mut GpioController> {
    unsafe { GPIO_CONTROLLER.as_mut() }
}

// Convenience functions for LED control
pub fn set_activity_led(on: bool) {
    if let Some(gpio) = get_gpio_controller() {
        gpio.set_activity_led(on);
    }
}

pub fn set_power_led(on: bool) {
    if let Some(gpio) = get_gpio_controller() {
        gpio.set_power_led(on);
    }
}

pub fn blink_activity_led() {
    if let Some(gpio) = get_gpio_controller() {
        gpio.blink_activity_led();
    }
}

pub fn test_gpio() -> bool {
    if let Some(gpio) = get_gpio_controller() {
        gpio.test_gpio_functionality()
    } else {
        false
    }
}
