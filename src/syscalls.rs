// System Call Interface for UNIX Compatibility
// POSIX-like system calls implementation

use crate::process::{PROCESS_MANAGER, Process, ProcessState};
use crate::filesystem::{read_file, write_file, create_file, file_exists};
use crate::uart::UART;
use heapless::{String, Vec};

const MAX_OPEN_FILES: usize = 32;
const MAX_FILENAME: usize = 64;

// System call numbers (Linux ARM64 compatible)
#[repr(u64)]
#[derive(Clone, Copy, Debug)]
pub enum SysCallNumber {
    Exit = 93,
    Fork = 57,
    Execve = 221,
    Open = 56,
    Close = 3,
    Read = 63,
    Write = 64,
    Getpid = 172,
    Getppid = 173,
    Kill = 129,
    Wait4 = 260,
    Pipe = 59,
    Dup = 23,
    Dup2 = 24,
    Chdir = 49,
    Getcwd = 79,
    Mkdir = 83,
    Rmdir = 84,
    Unlink = 87,
    Chmod = 90,
    Chown = 92,
    Utime = 132,
    Access = 21,
    Stat = 106,
    Lstat = 107,
    Fstat = 108,
}

// File descriptor structure
#[derive(Clone, Debug)]
pub struct FileDescriptor {
    pub fd: i32,
    pub path: String<MAX_FILENAME>,
    pub flags: u32,
    pub offset: usize,
    pub is_open: bool,
}

impl FileDescriptor {
    pub fn new(fd: i32, path: &str, flags: u32) -> Self {
        let mut path_str = String::new();
        let _ = path_str.push_str(path);
        
        Self {
            fd,
            path: path_str,
            flags,
            offset: 0,
            is_open: true,
        }
    }
}

// Process file descriptor table
pub struct ProcessFdTable {
    fds: Vec<FileDescriptor, MAX_OPEN_FILES>,
    next_fd: i32,
}

impl ProcessFdTable {
    pub fn new() -> Self {
        let mut table = Self {
            fds: Vec::new(),
            next_fd: 3, // Start after stdin(0), stdout(1), stderr(2)
        };
        
        // Initialize standard file descriptors
        let _ = table.fds.push(FileDescriptor::new(0, "/dev/stdin", 0));
        let _ = table.fds.push(FileDescriptor::new(1, "/dev/stdout", 1));
        let _ = table.fds.push(FileDescriptor::new(2, "/dev/stderr", 1));
        
        table
    }
    
    pub fn open_file(&mut self, path: &str, flags: u32) -> Result<i32, i32> {
        if self.fds.is_full() {
            return Err(-24); // EMFILE - Too many open files
        }
        
        let fd = self.next_fd;
        self.next_fd += 1;
        
        let file_desc = FileDescriptor::new(fd, path, flags);
        let _ = self.fds.push(file_desc);
        
        Ok(fd)
    }
    
    pub fn close_file(&mut self, fd: i32) -> Result<i32, i32> {
        for file_desc in &mut self.fds {
            if file_desc.fd == fd && file_desc.is_open {
                file_desc.is_open = false;
                return Ok(0);
            }
        }
        Err(-9) // EBADF - Bad file descriptor
    }
    
    pub fn get_fd(&self, fd: i32) -> Option<&FileDescriptor> {
        self.fds.iter().find(|f| f.fd == fd && f.is_open)
    }
    
    pub fn get_fd_mut(&mut self, fd: i32) -> Option<&mut FileDescriptor> {
        self.fds.iter_mut().find(|f| f.fd == fd && f.is_open)
    }
}

// Global file descriptor tables for each process (simplified)
static mut GLOBAL_FD_TABLE: ProcessFdTable = ProcessFdTable {
    fds: Vec::new(),
    next_fd: 3,
};

// System call handler
pub fn handle_syscall(syscall_num: u64, arg0: u64, arg1: u64, arg2: u64, _arg3: u64, _arg4: u64, _arg5: u64) -> i64 {
    match syscall_num {
        93 => sys_exit(arg0 as i32),
        57 => sys_fork(),
        56 => sys_open(arg0, arg1, arg2),
        63 => sys_read(arg0 as i32, arg1, arg2),
        64 => sys_write(arg0 as i32, arg1, arg2),
        172 => sys_getpid(),
        173 => sys_getppid(),
        129 => sys_kill(arg0 as i32, arg1 as i32),
        49 => sys_chdir(arg0),
        79 => sys_getcwd(arg0, arg1),
        83 => sys_mkdir(arg0, arg1),
        87 => sys_unlink(arg0),
        21 => sys_access(arg0, arg1),
        106 => sys_stat(arg0, arg1),
        _ => {
            UART.write_str("Unknown system call: ");
            UART.put_hex(syscall_num as u32);
            UART.write_str("\n");
            -38 // ENOSYS - Function not implemented
        }
    }
}

// System call implementations
fn sys_exit(status: i32) -> i64 {
    UART.write_str("Process exiting with status: ");
    UART.put_hex(status as u32);
    UART.write_str("\n");
    
    unsafe {
        let current_pid = PROCESS_MANAGER.current_pid();
        PROCESS_MANAGER.terminate_process(current_pid);
    }
    
    // This should not return
    loop {
        unsafe { core::arch::asm!("wfe"); }
    }
}

fn sys_fork() -> i64 {
    UART.write_str("fork() called\n");
    
    unsafe {
        let current_pid = PROCESS_MANAGER.current_pid();
        if let Some(parent) = PROCESS_MANAGER.get_process(current_pid) {
            let child_pid = PROCESS_MANAGER.create_process(parent.entry_point, current_pid);
            match child_pid {
                Some(pid) => pid as i64,
                None => -12, // ENOMEM - Out of memory
            }
        } else {
            -1 // Error
        }
    }
}

fn sys_open(pathname: u64, flags: u64, _mode: u64) -> i64 {
    // For simplicity, we'll use a fixed string for now
    let path = "/tmp/testfile"; // In real implementation, would read from pathname address
    
    UART.write_str("open() called: ");
    UART.write_str(path);
    UART.write_str("\n");
    
    unsafe {
        match GLOBAL_FD_TABLE.open_file(path, flags as u32) {
            Ok(fd) => fd as i64,
            Err(errno) => errno as i64,
        }
    }
}

fn sys_read(fd: i32, buf: u64, count: u64) -> i64 {
    UART.write_str("read() called, fd=");
    UART.put_hex(fd as u32);
    UART.write_str(", count=");
    UART.put_hex(count as u32);
    UART.write_str("\n");
    
    match fd {
        0 => { // stdin
            // For simplicity, return 0 (EOF)
            0
        }
        _ => {
            unsafe {
                if let Some(file_desc) = GLOBAL_FD_TABLE.get_fd(fd) {
                    // Read from virtual file system
                    if let Some(content) = read_file(file_desc.path.as_str()) {
                        let bytes_to_read = core::cmp::min(count as usize, content.len());
                        // In real implementation, would copy to buf address
                        bytes_to_read as i64
                    } else {
                        -2 // ENOENT - No such file or directory
                    }
                } else {
                    -9 // EBADF - Bad file descriptor
                }
            }
        }
    }
}

fn sys_write(fd: i32, buf: u64, count: u64) -> i64 {
    UART.write_str("write() called, fd=");
    UART.put_hex(fd as u32);
    UART.write_str(", count=");
    UART.put_hex(count as u32);
    UART.write_str("\n");
    
    match fd {
        1 | 2 => { // stdout/stderr
            // In real implementation, would read from buf address and write to UART
            UART.write_str("[STDOUT/STDERR output]\n");
            count as i64
        }
        _ => {
            unsafe {
                if let Some(_file_desc) = GLOBAL_FD_TABLE.get_fd(fd) {
                    // Write to virtual file system
                    // In real implementation, would read from buf address
                    count as i64
                } else {
                    -9 // EBADF - Bad file descriptor
                }
            }
        }
    }
}

fn sys_getpid() -> i64 {
    unsafe {
        PROCESS_MANAGER.current_pid() as i64
    }
}

fn sys_getppid() -> i64 {
    unsafe {
        let current_pid = PROCESS_MANAGER.current_pid();
        if let Some(process) = PROCESS_MANAGER.get_process(current_pid) {
            process.ppid as i64
        } else {
            1 // Return init process ID if not found
        }
    }
}

fn sys_kill(pid: i32, sig: i32) -> i64 {
    UART.write_str("kill() called, pid=");
    UART.put_hex(pid as u32);
    UART.write_str(", signal=");
    UART.put_hex(sig as u32);
    UART.write_str("\n");
    
    // Simplified signal handling - just terminate the process for now
    unsafe {
        if PROCESS_MANAGER.terminate_process(pid as u32) {
            0
        } else {
            -3 // ESRCH - No such process
        }
    }
}

fn sys_chdir(path: u64) -> i64 {
    // For simplicity, always succeed
    UART.write_str("chdir() called\n");
    0
}

fn sys_getcwd(buf: u64, size: u64) -> i64 {
    UART.write_str("getcwd() called\n");
    // In real implementation, would write current directory to buf
    // For now, just return success
    size as i64
}

fn sys_mkdir(pathname: u64, mode: u64) -> i64 {
    UART.write_str("mkdir() called\n");
    // In real implementation, would create directory in filesystem
    0
}

fn sys_unlink(pathname: u64) -> i64 {
    UART.write_str("unlink() called\n");
    // In real implementation, would remove file from filesystem
    0
}

fn sys_access(pathname: u64, mode: u64) -> i64 {
    UART.write_str("access() called\n");
    // In real implementation, would check file accessibility
    0
}

fn sys_stat(pathname: u64, statbuf: u64) -> i64 {
    UART.write_str("stat() called\n");
    // In real implementation, would fill stat structure
    0
}

// Initialize system call infrastructure
pub fn init_syscalls() {
    unsafe {
        GLOBAL_FD_TABLE = ProcessFdTable::new();
    }
    UART.write_str("System call interface initialized\n");
}
