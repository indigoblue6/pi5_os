// Extended UNIX Commands for POSIX Compatibility
// Additional commands to make the shell more UNIX-compatible

use crate::uart::UART;
use crate::process::{PROCESS_MANAGER, ProcessState};
use crate::filesystem;
use crate::users;
use crate::signals;
use crate::ipc;
use heapless::{String, Vec};

const MAX_ARGS: usize = 16;
const MAX_PATH: usize = 128;
const MAX_OUTPUT: usize = 512;

pub struct UnixCommands;

impl UnixCommands {
    // File operations
    pub fn cmd_touch(args: &Vec<&str, MAX_ARGS>) {
        if args.is_empty() {
            UART.write_str("touch: missing filename\n");
            return;
        }
        
        for &filename in args {
            if filesystem::file_exists(filename) {
                UART.write_str("touch: ");
                UART.write_str(filename);
                UART.write_str(" (file already exists, timestamp updated)\n");
            } else {
                if filesystem::create_file(filename, "") {
                    UART.write_str("touch: created ");
                    UART.write_str(filename);
                    UART.write_str("\n");
                } else {
                    UART.write_str("touch: cannot create ");
                    UART.write_str(filename);
                    UART.write_str("\n");
                }
            }
        }
    }
    
    pub fn cmd_rm(args: &Vec<&str, MAX_ARGS>) {
        if args.is_empty() {
            UART.write_str("rm: missing filename\n");
            return;
        }
        
        for &filename in args {
            if filesystem::file_exists(filename) {
                // Note: actual deletion would need to be implemented in filesystem module
                UART.write_str("rm: ");
                UART.write_str(filename);
                UART.write_str(" (deletion simulated)\n");
            } else {
                UART.write_str("rm: cannot remove '");
                UART.write_str(filename);
                UART.write_str("': No such file or directory\n");
            }
        }
    }
    
    pub fn cmd_cp(args: &Vec<&str, MAX_ARGS>) {
        if args.len() < 2 {
            UART.write_str("cp: missing file operand\n");
            UART.write_str("Usage: cp SOURCE DEST\n");
            return;
        }
        
        let source = args[0];
        let dest = args[1];
        
        if let Some(content) = filesystem::read_file(source) {
            if filesystem::create_file(dest, content.as_str()) {
                UART.write_str("cp: copied ");
                UART.write_str(source);
                UART.write_str(" to ");
                UART.write_str(dest);
                UART.write_str("\n");
            } else {
                UART.write_str("cp: cannot create ");
                UART.write_str(dest);
                UART.write_str("\n");
            }
        } else {
            UART.write_str("cp: cannot read ");
            UART.write_str(source);
            UART.write_str(": No such file or directory\n");
        }
    }
    
    pub fn cmd_mv(args: &Vec<&str, MAX_ARGS>) {
        if args.len() < 2 {
            UART.write_str("mv: missing file operand\n");
            UART.write_str("Usage: mv SOURCE DEST\n");
            return;
        }
        
        let source = args[0];
        let dest = args[1];
        
        UART.write_str("mv: ");
        UART.write_str(source);
        UART.write_str(" -> ");
        UART.write_str(dest);
        UART.write_str(" (move simulated)\n");
    }
    
    pub fn cmd_find(args: &Vec<&str, MAX_ARGS>) {
        let path = if args.is_empty() { "/" } else { args[0] };
        let pattern = if args.len() > 1 { args[1] } else { "*" };
        
        UART.write_str("find: searching in ");
        UART.write_str(path);
        UART.write_str(" for ");
        UART.write_str(pattern);
        UART.write_str("\n");
        
        // Simplified find - just list directory contents
        let entries = filesystem::list_directory(path);
        for file in entries {
            UART.write_str(file.name.as_str());
            UART.write_str("\n");
        }
    }
    
    pub fn cmd_grep(args: &Vec<&str, MAX_ARGS>) {
        if args.len() < 2 {
            UART.write_str("grep: missing pattern or file\n");
            UART.write_str("Usage: grep PATTERN FILE\n");
            return;
        }
        
        let pattern = args[0];
        let filename = args[1];
        
        if let Some(content) = filesystem::read_file(filename) {
            UART.write_str("grep: searching for '");
            UART.write_str(pattern);
            UART.write_str("' in ");
            UART.write_str(filename);
            UART.write_str("\n");
            
            // Simple pattern matching
            if content.contains(pattern) {
                UART.write_str("Found pattern in file\n");
            } else {
                UART.write_str("Pattern not found\n");
            }
        } else {
            UART.write_str("grep: ");
            UART.write_str(filename);
            UART.write_str(": No such file or directory\n");
        }
    }
    
    // Process management
    pub fn cmd_kill(args: &Vec<&str, MAX_ARGS>) {
        if args.is_empty() {
            UART.write_str("kill: missing PID\n");
            UART.write_str("Usage: kill [-SIGNAL] PID\n");
            return;
        }
        
        let mut signal = 15; // SIGTERM by default
        let mut pid_str = args[0];
        
        if args[0].starts_with('-') && args.len() > 1 {
            // Parse signal
            let signal_str = &args[0][1..];
            signal = match signal_str {
                "1" | "HUP" => 1,
                "2" | "INT" => 2,
                "9" | "KILL" => 9,
                "15" | "TERM" => 15,
                _ => {
                    UART.write_str("kill: invalid signal\n");
                    return;
                }
            };
            pid_str = args[1];
        }
        
        // Parse PID (simplified - assume it's a valid number)
        if let Some(pid) = Self::parse_number(pid_str) {
            UART.write_str("kill: sending signal ");
            UART.put_hex(signal as u32);
            UART.write_str(" to PID ");
            UART.put_hex(pid);
            UART.write_str("\n");
            
            let current_pid = unsafe { PROCESS_MANAGER.current_pid() };
            if let Err(e) = signals::send_signal(pid, signal, current_pid) {
                UART.write_str("kill: ");
                UART.write_str(e);
                UART.write_str("\n");
            }
        } else {
            UART.write_str("kill: invalid PID\n");
        }
    }
    
    pub fn cmd_jobs() {
        UART.write_str("Active jobs:\n");
        UART.write_str("  PID  STATE    COMMAND\n");
        UART.write_str("  ---  -----    -------\n");
        
        unsafe {
            for process in PROCESS_MANAGER.list_processes() {
                if process.state != ProcessState::Terminated {
                    UART.write_str("  ");
                    Self::print_number(process.pid, 3);
                    UART.write_str("  ");
                    
                    let state_str = match process.state {
                        ProcessState::Ready => "READY",
                        ProcessState::Running => "RUN  ",
                        ProcessState::Sleeping => "SLEEP",
                        ProcessState::Terminated => "TERM ",
                    };
                    UART.write_str(state_str);
                    UART.write_str("    process");
                    UART.write_str("\n");
                }
            }
        }
    }
    
    pub fn cmd_top() {
        UART.write_str("Top processes:\n");
        UART.write_str("  PID  PPID STATE    TIME COMMAND\n");
        UART.write_str("  ---  ---- -----    ---- -------\n");
        
        unsafe {
            for process in PROCESS_MANAGER.list_processes() {
                UART.write_str("  ");
                Self::print_number(process.pid, 3);
                UART.write_str("  ");
                Self::print_number(process.ppid, 4);
                UART.write_str(" ");
                
                let state_str = match process.state {
                    ProcessState::Ready => "READY",
                    ProcessState::Running => "RUN  ",
                    ProcessState::Sleeping => "SLEEP",
                    ProcessState::Terminated => "TERM ",
                };
                UART.write_str(state_str);
                UART.write_str("    ");
                Self::print_number(process.used_time, 4);
                UART.write_str(" process");
                UART.write_str("\n");
            }
        }
    }
    
    // User management
    pub fn cmd_whoami() {
        let (uid, _gid) = users::get_current_user();
        if let Some((username, _, _)) = users::get_user_info(uid) {
            UART.write_str(username.as_str());
            UART.write_str("\n");
        } else {
            UART.write_str("unknown\n");
        }
    }
    
    pub fn cmd_id(args: &Vec<&str, MAX_ARGS>) {
        let uid = if args.is_empty() {
            let (current_uid, _) = users::get_current_user();
            current_uid
        } else {
            if let Some(uid) = users::get_user_by_name(args[0]) {
                uid
            } else {
                UART.write_str("id: ");
                UART.write_str(args[0]);
                UART.write_str(": no such user\n");
                return;
            }
        };
        
        if let Some((username, gid, _)) = users::get_user_info(uid) {
            UART.write_str("uid=");
            UART.put_hex(uid);
            UART.write_str("(");
            UART.write_str(username.as_str());
            UART.write_str(") gid=");
            UART.put_hex(gid);
            
            let groups = users::get_user_groups(uid);
            if !groups.is_empty() {
                UART.write_str(" groups=");
                for (i, &group_gid) in groups.iter().enumerate() {
                    if i > 0 {
                        UART.write_str(",");
                    }
                    UART.put_hex(group_gid);
                }
            }
            UART.write_str("\n");
        }
    }
    
    pub fn cmd_su(args: &Vec<&str, MAX_ARGS>) {
        let target_user = if args.is_empty() { "root" } else { args[0] };
        
        if let Some(uid) = users::get_user_by_name(target_user) {
            UART.write_str("Password for ");
            UART.write_str(target_user);
            UART.write_str(": ");
            
            // In real implementation, would read password securely
            UART.write_str("(password input not implemented)\n");
            
            // For demo, just switch if current user is root
            if users::is_root() {
                if let Err(e) = users::switch_user(uid) {
                    UART.write_str("su: ");
                    UART.write_str(e);
                    UART.write_str("\n");
                }
            } else {
                UART.write_str("su: Authentication required\n");
            }
        } else {
            UART.write_str("su: user ");
            UART.write_str(target_user);
            UART.write_str(" does not exist\n");
        }
    }
    
    // System information
    pub fn cmd_uname(args: &Vec<&str, MAX_ARGS>) {
        let show_all = args.contains(&"-a");
        let show_kernel = args.is_empty() || args.contains(&"-s") || show_all;
        let show_nodename = args.contains(&"-n") || show_all;
        let show_release = args.contains(&"-r") || show_all;
        let show_version = args.contains(&"-v") || show_all;
        let show_machine = args.contains(&"-m") || show_all;
        
        if show_kernel {
            UART.write_str("Pi5OS");
            if !show_all && !args.contains(&"-s") {
                UART.write_str("\n");
                return;
            }
            UART.write_str(" ");
        }
        
        if show_nodename {
            UART.write_str("pi5-unix");
            UART.write_str(" ");
        }
        
        if show_release {
            UART.write_str("1.0.0");
            UART.write_str(" ");
        }
        
        if show_version {
            UART.write_str("#1");
            UART.write_str(" ");
        }
        
        if show_machine {
            UART.write_str("aarch64");
        }
        
        UART.write_str("\n");
    }
    
    pub fn cmd_uptime() {
        let uptime = crate::timer::get_uptime_seconds();
        UART.write_str(" ");
        Self::print_time(uptime);
        UART.write_str("  up ");
        
        let hours = uptime / 3600;
        let minutes = (uptime % 3600) / 60;
        
        if hours > 0 {
            Self::print_number(hours, 0);
            UART.write_str(" hours, ");
        }
        Self::print_number(minutes, 0);
        UART.write_str(" minutes\n");
    }
    
    pub fn cmd_free() {
        UART.write_str("             total       used       free     shared    buffers     cached\n");
        UART.write_str("Mem:       8388608     262144    8126464          0          0          0\n");
        UART.write_str("-/+ buffers/cache:     262144    8126464\n");
        UART.write_str("Swap:            0          0          0\n");
    }
    
    pub fn cmd_df() {
        UART.write_str("Filesystem     1K-blocks    Used Available Use% Mounted on\n");
        UART.write_str("rootfs               100      10        90  10% /\n");
        UART.write_str("proc                   0       0         0   0% /proc\n");
        UART.write_str("devfs                  0       0         0   0% /dev\n");
    }
    
    // Network-like commands (simulated)
    pub fn cmd_ping(args: &Vec<&str, MAX_ARGS>) {
        if args.is_empty() {
            UART.write_str("ping: missing host\n");
            return;
        }
        
        let host = args[0];
        UART.write_str("PING ");
        UART.write_str(host);
        UART.write_str(" (127.0.0.1): 56 data bytes\n");
        
        for i in 1..=4 {
            UART.write_str("64 bytes from ");
            UART.write_str(host);
            UART.write_str(": icmp_seq=");
            Self::print_number(i, 0);
            UART.write_str(" ttl=64 time=0.1 ms\n");
            
            // Small delay
            for _ in 0..1000000 {
                unsafe { core::arch::asm!("nop"); }
            }
        }
        
        UART.write_str("\n--- ");
        UART.write_str(host);
        UART.write_str(" ping statistics ---\n");
        UART.write_str("4 packets transmitted, 4 received, 0% packet loss\n");
    }
    
    // Text processing
    pub fn cmd_wc(args: &Vec<&str, MAX_ARGS>) {
        if args.is_empty() {
            UART.write_str("wc: missing filename\n");
            return;
        }
        
        for &filename in args {
            if let Some(content) = filesystem::read_file(filename) {
                let lines = content.matches('\n').count() + 1;
                let words = content.split_whitespace().count();
                let chars = content.len();
                
                Self::print_number(lines as u32, 7);
                UART.write_str(" ");
                Self::print_number(words as u32, 7);
                UART.write_str(" ");
                Self::print_number(chars as u32, 7);
                UART.write_str(" ");
                UART.write_str(filename);
                UART.write_str("\n");
            } else {
                UART.write_str("wc: ");
                UART.write_str(filename);
                UART.write_str(": No such file or directory\n");
            }
        }
    }
    
    pub fn cmd_head(args: &Vec<&str, MAX_ARGS>) {
        let lines = 10; // Default to 10 lines
        let filename = if args.is_empty() {
            UART.write_str("head: missing filename\n");
            return;
        } else {
            args[0]
        };
        
        if let Some(content) = filesystem::read_file(filename) {
            let mut line_count = 0;
            for line in content.split('\n') {
                if line_count >= lines {
                    break;
                }
                UART.write_str(line);
                UART.write_str("\n");
                line_count += 1;
            }
        } else {
            UART.write_str("head: ");
            UART.write_str(filename);
            UART.write_str(": No such file or directory\n");
        }
    }
    
    pub fn cmd_tail(args: &Vec<&str, MAX_ARGS>) {
        if args.is_empty() {
            UART.write_str("tail: missing filename\n");
            return;
        }
        
        let filename = args[0];
        if let Some(content) = filesystem::read_file(filename) {
            // Simplified tail - just show last few characters
            let start = if content.len() > 200 { content.len() - 200 } else { 0 };
            UART.write_str(&content[start..]);
            UART.write_str("\n");
        } else {
            UART.write_str("tail: ");
            UART.write_str(filename);
            UART.write_str(": No such file or directory\n");
        }
    }
    
    // IPC commands
    pub fn cmd_ipc() {
        let (pipes, msgqs, shms) = ipc::get_ipc_stats();
        UART.write_str("IPC Status:\n");
        UART.write_str("Pipes: ");
        Self::print_number(pipes as u32, 0);
        UART.write_str("\n");
        UART.write_str("Message Queues: ");
        Self::print_number(msgqs as u32, 0);
        UART.write_str("\n");
        UART.write_str("Shared Memory: ");
        Self::print_number(shms as u32, 0);
        UART.write_str("\n");
    }
    
    // Utility functions
    fn parse_number(s: &str) -> Option<u32> {
        let mut result = 0u32;
        for ch in s.chars() {
            if let Some(digit) = ch.to_digit(10) {
                result = result.wrapping_mul(10).wrapping_add(digit);
            } else {
                return None;
            }
        }
        Some(result)
    }
    
    fn print_number(num: u32, width: usize) {
        let mut buffer = [0u8; 10];
        let mut pos = 0;
        let mut n = num;
        
        if n == 0 {
            buffer[0] = b'0';
            pos = 1;
        } else {
            while n > 0 {
                buffer[pos] = b'0' + (n % 10) as u8;
                n /= 10;
                pos += 1;
            }
        }
        
        // Pad with spaces if needed
        for _ in pos..width {
            UART.write_char(' ');
        }
        
        // Print digits in reverse order
        for i in (0..pos).rev() {
            UART.write_char(buffer[i] as char);
        }
    }
    
    fn print_time(seconds: u32) {
        let hours = (seconds / 3600) % 24;
        let minutes = (seconds % 3600) / 60;
        let secs = seconds % 60;
        
        Self::print_number(hours, 2);
        UART.write_char(':');
        if minutes < 10 { UART.write_char('0'); }
        Self::print_number(minutes, 0);
        UART.write_char(':');
        if secs < 10 { UART.write_char('0'); }
        Self::print_number(secs, 0);
    }
}
