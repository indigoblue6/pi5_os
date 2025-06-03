// Basic Virtual File System for Minimal Pi5 OS
// Provides /proc, /dev, and basic file operations

use crate::uart::Uart;
use heapless::{String, Vec};

const MAX_FILES: usize = 32;
const MAX_FILENAME: usize = 64;
const MAX_CONTENT: usize = 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    RegularFile,
    Directory,
    Device,
    Proc,
}

#[derive(Debug, Clone)]
pub struct VirtualFile {
    pub name: String<MAX_FILENAME>,
    pub file_type: FileType,
    pub content: String<MAX_CONTENT>,
    pub size: usize,
    pub permissions: u32, // Unix-style permissions
}

impl VirtualFile {
    pub fn new(name: &str, file_type: FileType, content: &str) -> Self {
        let mut file_name = String::new();
        let _ = file_name.push_str(name);
        
        let mut file_content = String::new();
        let _ = file_content.push_str(content);
        
        let size = content.len();
        
        Self {
            name: file_name,
            file_type,
            content: file_content,
            size,
            permissions: match file_type {
                FileType::Directory => 0o755,
                FileType::Device => 0o666,
                FileType::Proc => 0o444,
                FileType::RegularFile => 0o644,
            },
        }
    }
}

pub struct VirtualFileSystem {
    files: Vec<VirtualFile, MAX_FILES>,
    uart: &'static mut Uart,
}

impl VirtualFileSystem {
    pub fn new(uart: &'static mut Uart) -> Self {
        let mut vfs = Self {
            files: Vec::new(),
            uart,
        };
        vfs.init_default_files();
        vfs
    }

    fn init_default_files(&mut self) {
        self.uart.write_str("Initializing virtual file system...\r\n");

        // Root directory
        self.add_file("/", FileType::Directory, "");
        
        // /proc directory and files
        self.add_file("/proc", FileType::Directory, "");
        self.add_file("/proc/version", FileType::Proc, 
                     "Minimal Pi5 OS version 0.1.0 (root@pi5) (aarch64) #1");
        self.add_file("/proc/cpuinfo", FileType::Proc,
                     "processor\t: 0\nBogoMIPS\t: 108.00\nFeatures\t: fp asimd evtstrm crc32 cpuid\nCPU implementer\t: 0x41\nCPU architecture: 8");
        self.add_file("/proc/meminfo", FileType::Proc,
                     "MemTotal:     8388608 kB\nMemFree:      7340032 kB\nMemAvailable: 7340032 kB");
        self.add_file("/proc/uptime", FileType::Proc, "");
        self.add_file("/proc/loadavg", FileType::Proc, "0.00 0.00 0.00 1/1 1");
        
        // /dev directory and devices
        self.add_file("/dev", FileType::Directory, "");
        self.add_file("/dev/null", FileType::Device, "");
        self.add_file("/dev/zero", FileType::Device, "");
        self.add_file("/dev/uart0", FileType::Device, "");
        self.add_file("/dev/mem", FileType::Device, "");
        
        // /sys directory for system information
        self.add_file("/sys", FileType::Directory, "");
        self.add_file("/sys/class", FileType::Directory, "");
        self.add_file("/sys/class/gpio", FileType::Directory, "");
        
        // /tmp directory
        self.add_file("/tmp", FileType::Directory, "");
        
        // Some example files
        self.add_file("/etc", FileType::Directory, "");
        self.add_file("/etc/hostname", FileType::RegularFile, "pi5-minimal");
        self.add_file("/etc/passwd", FileType::RegularFile, "root:x:0:0:root:/root:/bin/sh");

        self.uart.write_str("Virtual file system initialized\r\n");
    }

    fn add_file(&mut self, name: &str, file_type: FileType, content: &str) {
        if !self.files.is_full() {
            let file = VirtualFile::new(name, file_type, content);
            let _ = self.files.push(file);
        }
    }

    pub fn list_directory(&self, path: &str) -> Vec<&VirtualFile, MAX_FILES> {
        let mut entries = Vec::new();
        
        // Normalize path
        let normalized_path = if path == "/" { "" } else { path };
        
        for file in &self.files {
            let file_path = file.name.as_str();
            
            if path == "/" {
                // Root directory - show top-level entries
                if file_path != "/" && !file_path.contains('/') || 
                   (file_path.starts_with('/') && file_path[1..].chars().filter(|&c| c == '/').count() == 0) {
                    if !entries.is_full() {
                        let _ = entries.push(file);
                    }
                }
            } else {
                // Show direct children of the specified directory
                if file_path.starts_with(normalized_path) && file_path != normalized_path {
                    let suffix = &file_path[normalized_path.len()..];
                    if suffix.starts_with('/') {
                        let remaining = &suffix[1..];
                        if !remaining.contains('/') && !entries.is_full() {
                            let _ = entries.push(file);
                        }
                    }
                }
            }
        }
        
        entries
    }

    pub fn read_file(&mut self, path: &str) -> Option<String<MAX_CONTENT>> {
        // Handle dynamic files
        match path {
            "/proc/uptime" => {
                let uptime = crate::timer::get_uptime_seconds();
                let mut content = String::new();
                self.format_number(&mut content, uptime);
                let _ = content.push_str(".00 ");
                self.format_number(&mut content, uptime);
                let _ = content.push_str(".00");
                return Some(content);
            }
            _ => {}
        }

        // Handle static files
        for file in &self.files {
            if file.name.as_str() == path {
                return Some(file.content.clone());
            }
        }
        
        None
    }

    pub fn file_exists(&self, path: &str) -> bool {
        for file in &self.files {
            if file.name.as_str() == path {
                return true;
            }
        }
        false
    }

    pub fn get_file_info(&self, path: &str) -> Option<&VirtualFile> {
        for file in &self.files {
            if file.name.as_str() == path {
                return Some(file);
            }
        }
        None
    }

    pub fn create_file(&mut self, path: &str, content: &str) -> bool {
        if self.file_exists(path) {
            return false; // File already exists
        }
        
        if !self.files.is_full() {
            let file = VirtualFile::new(path, FileType::RegularFile, content);
            let _ = self.files.push(file);
            true
        } else {
            false // No space
        }
    }

    pub fn write_file(&mut self, path: &str, content: &str) -> bool {
        for file in &mut self.files {
            if file.name.as_str() == path && file.file_type != FileType::Proc {
                file.content.clear();
                let _ = file.content.push_str(content);
                file.size = content.len();
                return true;
            }
        }
        false
    }

    pub fn delete_file(&mut self, path: &str) -> bool {
        for (i, file) in self.files.iter().enumerate() {
            if file.name.as_str() == path && file.file_type == FileType::RegularFile {
                self.files.remove(i);
                return true;
            }
        }
        false
    }

    fn format_number(&self, string: &mut String<MAX_CONTENT>, num: u32) {
        let mut buffer = [0u8; 10];
        let mut pos = 0;
        let mut n = num;
        
        if n == 0 {
            let _ = string.push('0');
            return;
        }
        
        while n > 0 {
            buffer[pos] = b'0' + (n % 10) as u8;
            n /= 10;
            pos += 1;
        }
        
        // Add digits in reverse order
        for i in (0..pos).rev() {
            let _ = string.push(buffer[i] as char);
        }
    }

    pub fn get_stats(&self) -> (usize, usize) {
        let used = self.files.len();
        let total = MAX_FILES;
        (used, total)
    }
}

// Global file system instance
static mut VFS: Option<VirtualFileSystem> = None;

pub fn init_filesystem(uart: &'static mut Uart) -> Result<(), &'static str> {
    unsafe {
        VFS = Some(VirtualFileSystem::new(uart));
    }
    Ok(())
}

pub fn get_filesystem() -> Option<&'static mut VirtualFileSystem> {
    unsafe { VFS.as_mut() }
}

// Convenience functions
pub fn list_directory(path: &str) -> Vec<&'static VirtualFile, MAX_FILES> {
    if let Some(vfs) = get_filesystem() {
        vfs.list_directory(path)
    } else {
        Vec::new()
    }
}

pub fn read_file(path: &str) -> Option<String<MAX_CONTENT>> {
    if let Some(vfs) = get_filesystem() {
        vfs.read_file(path)
    } else {
        None
    }
}

pub fn file_exists(path: &str) -> bool {
    if let Some(vfs) = get_filesystem() {
        vfs.file_exists(path)
    } else {
        false
    }
}

pub fn create_file(path: &str, content: &str) -> bool {
    if let Some(vfs) = get_filesystem() {
        vfs.create_file(path, content)
    } else {
        false
    }
}

pub fn write_file(path: &str, content: &str) -> bool {
    if let Some(vfs) = get_filesystem() {
        vfs.write_file(path, content)
    } else {
        false
    }
}

pub fn get_file_info(path: &str) -> Option<&'static VirtualFile> {
    if let Some(vfs) = get_filesystem() {
        vfs.get_file_info(path)
    } else {
        None
    }
}
