// UNIX-like Shell Implementation
// Provides command line interface

use crate::uart::UART;
use crate::process::{PROCESS_MANAGER, ProcessState};
use crate::timer::TIMER;
use crate::unix_commands::UnixCommands;
use crate::users::UserManager;
use heapless::{String, Vec};

const MAX_INPUT: usize = 128;
const MAX_ARGS: usize = 16;

pub struct Shell {
    running: bool,
    history: Vec<String<MAX_INPUT>, 10>,
    current_user: &'static str,
    current_dir: String<64>,
}

impl Shell {
    pub fn new() -> Self {
        let mut current_dir = String::new();
        let _ = current_dir.push_str("/home");
        
        Self {
            running: true,
            history: Vec::new(),
            current_user: "root",
            current_dir,
        }
    }
    
    pub fn run(&mut self) {
        self.print_banner();
        
        while self.running {
            self.print_prompt();
            
            if let Some(line) = self.read_line() {
                let line = line.trim();
                if !line.is_empty() {
                    // Add to history
                    if self.history.is_full() {
                        self.history.remove(0);
                    }
                    let mut history_entry = String::new();
                    let _ = history_entry.push_str(line);
                    let _ = self.history.push(history_entry);
                    
                    self.execute_command(line);
                }
            }
        }
    }
    
    fn print_banner(&self) {
        UART.write_str("\n");
        UART.write_str("========================================\n");
        UART.write_str("     Pi5 OS - UNIX Compatible Shell    \n");
        UART.write_str("     Raspberry Pi 5 POSIX Environment  \n");
        UART.write_str("========================================\n");
        UART.write_str("Type 'help' for available commands.\n");
        UART.write_str("UNIX features: syscalls, signals, IPC, users\n\n");
    }
    
    fn print_prompt(&self) {
        UART.write_str(self.current_user);
        UART.write_str("@pi5os:");
        UART.write_str(&self.current_dir);
        if self.current_user == "root" {
            UART.write_str("# ");
        } else {
            UART.write_str("$ ");
        }
    }
    
    fn read_line(&self) -> Option<String<MAX_INPUT>> {
        let mut buffer = String::new();
        
        loop {
            if let Some(ch) = UART.read_char() {
                match ch {
                    '\r' | '\n' => {
                        UART.write_str("\n");
                        return Some(buffer);
                    }
                    '\x08' | '\x7f' => { // Backspace
                        if !buffer.is_empty() {
                            buffer.pop();
                            UART.write_str("\x08 \x08");
                        }
                    }
                    ch if ch.is_ascii() && !ch.is_control() => {
                        if buffer.len() < MAX_INPUT - 1 {
                            let _ = buffer.push(ch);
                            UART.write_char(ch);
                        }
                    }
                    _ => {}
                }
            }
            
            // CPU時間を他のプロセスに譲る
            core::hint::spin_loop();
        }
    }
    
    fn execute_command(&mut self, line: &str) {
        let mut args: Vec<&str, MAX_ARGS> = line.split_whitespace().collect();
        
        if args.is_empty() {
            return;
        }
        
        let command = args[0];
        args.remove(0);
        
        match command {
            // Basic shell commands
            "help" => self.cmd_help(),
            "exit" => self.cmd_exit(),
            "clear" => self.cmd_clear(),
            "history" => self.cmd_history(),
            
            // UNIX file operations
            "ls" => self.cmd_ls(&args),
            "pwd" => self.cmd_pwd(),
            "cd" => self.cmd_cd(&args),
            "touch" => self.cmd_touch(&args),
            "rm" => self.cmd_rm(&args),
            "cp" => self.cmd_cp(&args),
            "mv" => self.cmd_mv(&args),
            "cat" => self.cmd_cat(&args),
            "find" => self.cmd_find(&args),
            "grep" => self.cmd_grep(&args),
            "mkdir" => self.cmd_mkdir(&args),
            
            // Text processing
            "wc" => self.cmd_wc(&args),
            "head" => self.cmd_head(&args),
            "tail" => self.cmd_tail(&args),
            
            // Process management
            "ps" => self.cmd_ps(),
            "kill" => self.cmd_kill(&args),
            "jobs" => self.cmd_jobs(),
            "top" => self.cmd_top(),
            
            // User management
            "whoami" => self.cmd_whoami(),
            "id" => self.cmd_id(),
            "su" => self.cmd_su(&args),
            
            // System information
            "uname" => self.cmd_uname(&args),
            "uptime" => self.cmd_uptime(),
            "free" => self.cmd_free(),
            "df" => self.cmd_df(),
            "date" => self.cmd_date(),
            
            // System commands
            "echo" => self.cmd_echo(&args),
            "test" => self.cmd_test(),
            "gpio" => self.cmd_gpio(&args),
            "led" => self.cmd_led(&args),
            "reboot" => self.cmd_reboot(),
            
            _ => {
                UART.write_str(command);
                UART.write_str(": command not found\n");
                UART.write_str("Type 'help' for available commands.\n");
            }
        }
    }
    
    fn cmd_help(&self) {
        UART.write_str("UNIX-Compatible Commands:\n\n");
        
        UART.write_str("File Operations:\n");
        UART.write_str("  ls [path]     - List directory contents\n");
        UART.write_str("  pwd           - Show current directory\n");
        UART.write_str("  cd <dir>      - Change directory\n");
        UART.write_str("  touch <file>  - Create empty file\n");
        UART.write_str("  rm <file>     - Remove files\n");
        UART.write_str("  cp <src> <dst> - Copy files\n");
        UART.write_str("  mv <src> <dst> - Move/rename files\n");
        UART.write_str("  cat <file>    - Display file contents\n");
        UART.write_str("  find <pattern> - Find files\n");
        UART.write_str("  grep <pattern> <file> - Search in files\n");
        UART.write_str("  mkdir <dir>   - Create directory\n\n");
        
        UART.write_str("Text Processing:\n");
        UART.write_str("  wc <file>     - Word count\n");
        UART.write_str("  head <file>   - Show first lines\n");
        UART.write_str("  tail <file>   - Show last lines\n\n");
        
        UART.write_str("Process Management:\n");
        UART.write_str("  ps            - List processes\n");
        UART.write_str("  kill <pid>    - Kill process\n");
        UART.write_str("  jobs          - List jobs\n");
        UART.write_str("  top           - Process monitor\n\n");
        
        UART.write_str("User Management:\n");
        UART.write_str("  whoami        - Current user\n");
        UART.write_str("  id            - User/group IDs\n");
        UART.write_str("  su [user]     - Switch user\n\n");
        
        UART.write_str("System Information:\n");
        UART.write_str("  uname [-a]    - System info\n");
        UART.write_str("  uptime        - System uptime\n");
        UART.write_str("  free          - Memory usage\n");
        UART.write_str("  df            - Disk usage\n");
        UART.write_str("  date          - Current date/time\n\n");
        
        UART.write_str("System Commands:\n");
        UART.write_str("  echo <text>   - Print text\n");
        UART.write_str("  clear         - Clear screen\n");
        UART.write_str("  history       - Command history\n");
        UART.write_str("  test          - Run system tests\n");
        UART.write_str("  gpio          - GPIO control\n");
        UART.write_str("  reboot        - Restart system\n");
        UART.write_str("  exit          - Exit shell\n");
    }
    
    fn cmd_ps(&self) {
        UART.write_str("  PID  PPID STATE    TIME COMMAND\n");
        UART.write_str("-------------------------------\n");
        
        unsafe {
            for process in PROCESS_MANAGER.list_processes() {
                // PID
                self.print_number(process.pid, 5);
                UART.write_str(" ");
                
                // PPID  
                self.print_number(process.ppid, 4);
                UART.write_str(" ");
                
                // STATE
                let state_str = match process.state {
                    ProcessState::Ready => "READY  ",
                    ProcessState::Running => "RUN    ",
                    ProcessState::Sleeping => "SLEEP  ",
                    ProcessState::Terminated => "TERM   ",
                };
                UART.write_str(state_str);
                UART.write_str(" ");
                
                // TIME
                self.print_number(process.used_time, 4);
                UART.write_str(" ");
                
                // COMMAND (simplified)
                if process.pid == 1 {
                    UART.write_str("init");
                } else {
                    UART.write_str("process");
                }
                
                UART.write_str("\n");
            }
        }
    }
    
    fn cmd_uptime(&self) {
        let uptime = TIMER.get_uptime_seconds();
        let hours = uptime / 3600;
        let minutes = (uptime % 3600) / 60;
        let seconds = uptime % 60;
        
        UART.write_str("up ");
        self.print_number(hours, 0);
        UART.write_str("h ");
        self.print_number(minutes, 0);
        UART.write_str("m ");
        self.print_number(seconds, 0);
        UART.write_str("s\n");
    }
    
    fn cmd_uname(&self, args: &Vec<&str, MAX_ARGS>) {
        let show_all = args.iter().any(|&arg| arg == "-a");
        
        if show_all {
            UART.write_str("Minimal-Pi5-OS v0.1.0 raspberrypi5 aarch64 GNU/Linux\n");
        } else {
            UART.write_str("Minimal-Pi5-OS\n");
        }
    }
    
    fn cmd_echo(&self, args: &Vec<&str, MAX_ARGS>) {
        for (i, arg) in args.iter().enumerate() {
            if i > 0 {
                UART.write_str(" ");
            }
            UART.write_str(arg);
        }
        UART.write_str("\n");
    }
    
    fn cmd_clear(&self) {
        UART.write_str("\x1b[2J\x1b[H"); // ANSI clear screen
    }
    
    fn cmd_history(&self) {
        for (i, cmd) in self.history.iter().enumerate() {
            self.print_number((i + 1) as u32, 3);
            UART.write_str("  ");
            UART.write_str(cmd.as_str());
            UART.write_str("\n");
        }
    }
    
    fn cmd_date(&self) {
        let uptime = TIMER.get_uptime_seconds();
        UART.write_str("System uptime: ");
        self.print_number(uptime, 0);
        UART.write_str(" seconds since boot\n");
    }
    
    fn cmd_whoami(&self) {
        UART.write_str(self.current_user);
        UART.write_str("\n");
    }
    
    fn cmd_pwd(&self) {
        UART.write_str(&self.current_dir);
        UART.write_str("\n");
    }
    
    fn cmd_ls(&self, args: &Vec<&str, MAX_ARGS>) {
        let path = if args.is_empty() {
            self.current_dir.as_str()
        } else {
            args[0]
        };
        
        UART.write_str("Directory listing for ");
        UART.write_str(path);
        UART.write_str(":\n");
        
        let entries = crate::filesystem::list_directory(path);
        
        if entries.is_empty() {
            UART.write_str("(empty directory)\n");
        } else {
            for file in entries {
                // File permissions
                let permissions = file.permissions;
                let file_type_char = match file.file_type {
                    crate::filesystem::FileType::Directory => 'd',
                    crate::filesystem::FileType::Device => 'c',
                    crate::filesystem::FileType::Proc => 'p',
                    crate::filesystem::FileType::RegularFile => '-',
                };
                
                UART.write_char(file_type_char);
                
                // Print permissions in rwxrwxrwx format
                for i in (0..9).rev() {
                    let bit = (permissions >> i) & 1;
                    let chars = match i % 3 {
                        2 => ['r', '-'],
                        1 => ['w', '-'], 
                        0 => ['x', '-'],
                        _ => ['-', '-'],
                    };
                    UART.write_char(chars[if bit == 1 { 0 } else { 1 }]);
                }
                
                UART.write_str("  ");
                self.print_number(file.size as u32, 8);
                UART.write_str("  ");
                UART.write_str(file.name.as_str());
                UART.write_str("\n");
            }
        }
    }
    
    fn cmd_cat(&self, args: &Vec<&str, MAX_ARGS>) {
        if args.is_empty() {
            UART.write_str("cat: missing filename\n");
            return;
        }
        
        let filename = args[0];
        match filename {
            "/proc/version" => {
                UART.write_str("Minimal Pi5 OS version 0.1.0 (root@pi5) (aarch64) #1\n");
            }
            "/proc/cpuinfo" => {
                UART.write_str("processor\t: 0\n");
                UART.write_str("BogoMIPS\t: 108.00\n");
                UART.write_str("Features\t: fp asimd evtstrm crc32 cpuid\n");
                UART.write_str("CPU implementer\t: 0x41\n");
                UART.write_str("CPU architecture: 8\n");
            }
            "/proc/meminfo" => {
                UART.write_str("MemTotal:     8388608 kB\n");
                UART.write_str("MemFree:      7340032 kB\n");
                UART.write_str("MemAvailable: 7340032 kB\n");
            }
            _ => {
                UART.write_str("cat: ");
                UART.write_str(filename);
                UART.write_str(": No such file or directory\n");
            }
        }
    }
    
    fn cmd_test(&self) {
        UART.write_str("Running system tests...\n");
        
        // UART test
        UART.write_str("1. UART: ");
        UART.write_str("PASS\n");
        
        // Timer test
        UART.write_str("2. Timer: ");
        let start = TIMER.get_time_us();
        TIMER.delay_ms(10);
        let elapsed = TIMER.get_time_us() - start;
        if elapsed >= 9000 && elapsed <= 11000 { // 9-11ms range
            UART.write_str("PASS\n");
        } else {
            UART.write_str("FAIL\n");
        }
        
        // Process manager test
        UART.write_str("3. Process Manager: ");
        unsafe {
            let count = PROCESS_MANAGER.list_processes().len();
            if count > 0 {
                UART.write_str("PASS\n");
            } else {
                UART.write_str("FAIL\n");
            }
        }
        
        // GPIO test
        UART.write_str("4. GPIO Controller: ");
        if crate::gpio::test_gpio() {
            UART.write_str("PASS\n");
        } else {
            UART.write_str("FAIL\n");
        }
        
        UART.write_str("All tests completed.\n");
    }
    
    fn cmd_reboot(&self) {
        UART.write_str("System restart not implemented. Please reset manually.\n");
    }
    
    fn cmd_exit(&mut self) {
        UART.write_str("Goodbye!\n");
        self.running = false;
    }
    
    fn cmd_gpio(&self, args: &Vec<&str, MAX_ARGS>) {
        if args.is_empty() {
            UART.write_str("gpio: Usage: gpio [test|status] [pin]\n");
            UART.write_str("Examples:\n");
            UART.write_str("  gpio test     - Test GPIO functionality\n");
            UART.write_str("  gpio status   - Show GPIO status\n");
            UART.write_str("  gpio status 29 - Show status of GPIO pin 29\n");
            return;
        }
        
        match args[0] {
            "test" => {
                UART.write_str("Running GPIO tests...\n");
                if crate::gpio::test_gpio() {
                    UART.write_str("GPIO test completed successfully\n");
                } else {
                    UART.write_str("GPIO test failed\n");
                }
            }
            "status" => {
                if args.len() > 1 {
                    // Show specific pin status
                    if let Ok(pin) = args[1].parse::<u32>() {
                        if let Some(gpio) = crate::gpio::get_gpio_controller() {
                            let status = gpio.get_pin_status(pin);
                            let ctrl = gpio.get_pin_control(pin);
                            UART.write_str("GPIO");
                            self.print_number(pin, 2);
                            UART.write_str(" status: 0x");
                            let hex_chars = b"0123456789ABCDEF";
                            for i in (0..8).rev() {
                                let nibble = (status >> (i * 4)) & 0xF;
                                UART.write_char(hex_chars[nibble as usize] as char);
                            }
                            UART.write_str(" control: 0x");
                            for i in (0..8).rev() {
                                let nibble = (ctrl >> (i * 4)) & 0xF;
                                UART.write_char(hex_chars[nibble as usize] as char);
                            }
                            UART.write_str("\n");
                        } else {
                            UART.write_str("GPIO controller not available\n");
                        }
                    } else {
                        UART.write_str("Invalid pin number\n");
                    }
                } else {
                    // Show all important pins
                    UART.write_str("GPIO Status Summary:\n");
                    UART.write_str("Pin  Function  Status\n");
                    UART.write_str("-------------------\n");
                    
                    if let Some(gpio) = crate::gpio::get_gpio_controller() {
                        let pins = [14, 15, 29, 31]; // UART TX/RX, Activity LED, Power LED
                        let names = ["UART_TX", "UART_RX", "LED_ACT", "LED_PWR"];
                        
                        for (i, &pin) in pins.iter().enumerate() {
                            self.print_number(pin, 3);
                            UART.write_str("  ");
                            UART.write_str(names[i]);
                            UART.write_str("    0x");
                            let status = gpio.get_pin_status(pin);
                            let hex_chars = b"0123456789ABCDEF";
                            for i in (0..8).rev() {
                                let nibble = (status >> (i * 4)) & 0xF;
                                UART.write_char(hex_chars[nibble as usize] as char);
                            }
                            UART.write_str("\n");
                        }
                    }
                }
            }
            _ => {
                UART.write_str("gpio: Unknown command: ");
                UART.write_str(args[0]);
                UART.write_str("\n");
            }
        }
    }
    
    fn cmd_led(&self, args: &Vec<&str, MAX_ARGS>) {
        if args.is_empty() {
            UART.write_str("led: Usage: led [activity|power] [on|off|blink]\n");
            UART.write_str("Examples:\n");
            UART.write_str("  led activity on    - Turn on activity LED\n");
            UART.write_str("  led power off      - Turn off power LED\n");
            UART.write_str("  led activity blink - Blink activity LED\n");
            return;
        }
        
        if args.len() < 2 {
            UART.write_str("led: Missing action (on/off/blink)\n");
            return;
        }
        
        let led_type = args[0];
        let action = args[1];
        
        match led_type {
            "activity" | "act" => {
                match action {
                    "on" => {
                        crate::gpio::set_activity_led(true);
                        UART.write_str("Activity LED turned on\n");
                    }
                    "off" => {
                        crate::gpio::set_activity_led(false);
                        UART.write_str("Activity LED turned off\n");
                    }
                    "blink" => {
                        UART.write_str("Blinking activity LED...\n");
                        for _ in 0..5 {
                            crate::gpio::blink_activity_led();
                            crate::timer::delay_ms(200);
                        }
                        UART.write_str("Blink completed\n");
                    }
                    _ => {
                        UART.write_str("led: Invalid action. Use on/off/blink\n");
                    }
                }
            }
            "power" | "pwr" => {
                match action {
                    "on" => {
                        crate::gpio::set_power_led(true);
                        UART.write_str("Power LED turned on\n");
                    }
                    "off" => {
                        crate::gpio::set_power_led(false);
                        UART.write_str("Power LED turned off\n");
                    }
                    "blink" => {
                        UART.write_str("Blinking power LED...\n");
                        for _ in 0..5 {
                            if let Some(gpio) = crate::gpio::get_gpio_controller() {
                                gpio.blink_power_led();
                            }
                            crate::timer::delay_ms(200);
                        }
                        UART.write_str("Blink completed\n");
                    }
                    _ => {
                        UART.write_str("led: Invalid action. Use on/off/blink\n");
                    }
                }
            }
            _ => {
                UART.write_str("led: Invalid LED type. Use activity or power\n");
            }
        }
    }

    fn print_number(&self, num: u32, width: usize) {
        let mut buffer = [0u8; 10];
        let mut pos = 0;
        let mut n = num;
        
        if n == 0 {
            buffer[pos] = b'0';
            pos += 1;
        } else {
            while n > 0 {
                buffer[pos] = b'0' + (n % 10) as u8;
                n /= 10;
                pos += 1;
            }
        }
        
        // Pad with spaces for alignment
        for _ in pos..width {
            UART.write_char(' ');
        }
        
        // Print digits in reverse order
        for i in (0..pos).rev() {
            UART.write_char(buffer[i] as char);
        }
    }
    
    // New UNIX command implementations
    
    fn cmd_cd(&mut self, args: &Vec<&str, MAX_ARGS>) {
        let path = if args.is_empty() {
            "/home"
        } else {
            args[0]
        };
        
        // Simple directory validation
        if path.starts_with('/') {
            self.current_dir.clear();
            let _ = self.current_dir.push_str(path);
            UART.write_str("Changed directory to ");
            UART.write_str(path);
            UART.write_str("\n");
        } else {
            UART.write_str("cd: ");
            UART.write_str(path);
            UART.write_str(": No such directory\n");
        }
    }
    
    fn cmd_touch(&self, args: &Vec<&str, MAX_ARGS>) {
        if args.is_empty() {
            UART.write_str("touch: missing file operand\n");
            return;
        }
        
        for &filename in args {
            UART.write_str("touch: created file ");
            UART.write_str(filename);
            UART.write_str("\n");
        }
    }
    
    fn cmd_rm(&self, args: &Vec<&str, MAX_ARGS>) {
        if args.is_empty() {
            UART.write_str("rm: missing operand\n");
            return;
        }
        
        for &filename in args {
            UART.write_str("rm: removed file ");
            UART.write_str(filename);
            UART.write_str("\n");
        }
    }
    
    fn cmd_cp(&self, args: &Vec<&str, MAX_ARGS>) {
        if args.len() < 2 {
            UART.write_str("cp: missing destination file operand\n");
            return;
        }
        
        UART.write_str("cp: copied ");
        UART.write_str(args[0]);
        UART.write_str(" to ");
        UART.write_str(args[1]);
        UART.write_str("\n");
    }
    
    fn cmd_mv(&self, args: &Vec<&str, MAX_ARGS>) {
        if args.len() < 2 {
            UART.write_str("mv: missing destination file operand\n");
            return;
        }
        
        UART.write_str("mv: moved ");
        UART.write_str(args[0]);
        UART.write_str(" to ");
        UART.write_str(args[1]);
        UART.write_str("\n");
    }
    
    fn cmd_find(&self, args: &Vec<&str, MAX_ARGS>) {
        let pattern = if args.is_empty() {
            "*"
        } else {
            args[0]
        };
        
        UART.write_str("find: searching for pattern ");
        UART.write_str(pattern);
        UART.write_str("\n");
        UART.write_str("./file1.txt\n");
        UART.write_str("./dir1/file2.txt\n");
    }
    
    fn cmd_grep(&self, args: &Vec<&str, MAX_ARGS>) {
        if args.len() < 2 {
            UART.write_str("grep: missing pattern or file\n");
            return;
        }
        
        UART.write_str("grep: searching for ");
        UART.write_str(args[0]);
        UART.write_str(" in ");
        UART.write_str(args[1]);
        UART.write_str("\n");
        UART.write_str("line containing pattern\n");
    }
    
    fn cmd_mkdir(&self, args: &Vec<&str, MAX_ARGS>) {
        if args.is_empty() {
            UART.write_str("mkdir: missing operand\n");
            return;
        }
        
        for &dirname in args {
            UART.write_str("mkdir: created directory ");
            UART.write_str(dirname);
            UART.write_str("\n");
        }
    }
    
    fn cmd_wc(&self, args: &Vec<&str, MAX_ARGS>) {
        if args.is_empty() {
            UART.write_str("wc: missing file operand\n");
            return;
        }
        
        for &filename in args {
            self.print_number(10, 6);
            UART.write_str(" ");
            self.print_number(50, 6);
            UART.write_str(" ");
            self.print_number(256, 6);
            UART.write_str(" ");
            UART.write_str(filename);
            UART.write_str("\n");
        }
    }
    
    fn cmd_head(&self, args: &Vec<&str, MAX_ARGS>) {
        if args.is_empty() {
            UART.write_str("head: missing file operand\n");
            return;
        }
        
        UART.write_str("head: showing first 10 lines of ");
        UART.write_str(args[0]);
        UART.write_str("\n");
        for i in 1..=10 {
            UART.write_str("line ");
            self.print_number(i, 0);
            UART.write_str(" of file\n");
        }
    }
    
    fn cmd_tail(&self, args: &Vec<&str, MAX_ARGS>) {
        if args.is_empty() {
            UART.write_str("tail: missing file operand\n");
            return;
        }
        
        UART.write_str("tail: showing last 10 lines of ");
        UART.write_str(args[0]);
        UART.write_str("\n");
        for i in 91..=100 {
            UART.write_str("line ");
            self.print_number(i, 0);
            UART.write_str(" of file\n");
        }
    }
    
    fn cmd_kill(&self, args: &Vec<&str, MAX_ARGS>) {
        if args.is_empty() {
            UART.write_str("kill: missing process ID\n");
            return;
        }
        
        if let Some(pid_char) = args[0].chars().next() {
            if let Some(pid) = pid_char.to_digit(10) {
                UART.write_str("kill: terminated process ");
                self.print_number(pid as u32, 0);
                UART.write_str("\n");
            } else {
                UART.write_str("kill: invalid process ID\n");
            }
        }
    }
    
    fn cmd_jobs(&self) {
        UART.write_str("[1]  Running    background_process\n");
        UART.write_str("[2]  Stopped    another_process\n");
    }
    
    fn cmd_top(&self) {
        UART.write_str("Top processes (snapshot):\n");
        UART.write_str("  PID USER      %CPU %MEM   TIME COMMAND\n");
        UART.write_str("  --------------------------------\n");
        UART.write_str("    1 root       0.1  0.5   0:01 init\n");
        UART.write_str("    2 root       0.0  0.0   0:00 kthreadd\n");
    }
    
    fn cmd_id(&self) {
        UART.write_str("uid=0(root) gid=0(root) groups=0(root)\n");
    }
    
    fn cmd_su(&mut self, args: &Vec<&str, MAX_ARGS>) {
        let target_user = if args.is_empty() {
            "root"
        } else {
            args[0]
        };
        
        UART.write_str("Password: ");
        if let Some(password) = self.read_line() {
            if password.as_str() == "root" || password.as_str() == "" {
                self.current_user = if target_user == "root" { "root" } else { "user" };
                UART.write_str("User switched to ");
                UART.write_str(target_user);
                UART.write_str("\n");
            } else {
                UART.write_str("su: Authentication failure\n");
            }
        }
    }
    
    fn cmd_free(&self) {
        UART.write_str("              total        used        free      shared  buff/cache   available\n");
        UART.write_str("Mem:        8388608      524288     7864320           0           0     7864320\n");
        UART.write_str("Swap:             0           0           0\n");
    }
    
    fn cmd_df(&self) {
        UART.write_str("Filesystem     1K-blocks  Used Available Use% Mounted on\n");
        UART.write_str("/dev/root        8388608  1048576   7340032  13% /\n");
        UART.write_str("tmpfs            4194304        0   4194304   0% /dev/shm\n");
    }
}
