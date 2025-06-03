// Inter-Process Communication (IPC) for UNIX Compatibility
// Pipes, message queues, and shared memory implementation

use crate::uart::UART;
use heapless::{String, Vec, FnvIndexMap};

const MAX_PIPES: usize = 32;
const PIPE_BUFFER_SIZE: usize = 4096;
const MAX_MESSAGE_QUEUES: usize = 16;
const MAX_MESSAGE_SIZE: usize = 1024;
const MAX_MESSAGES_PER_QUEUE: usize = 16;

// Pipe implementation
#[derive(Debug, Clone)]
pub struct Pipe {
    pub read_fd: i32,
    pub write_fd: i32,
    pub buffer: Vec<u8, PIPE_BUFFER_SIZE>,
    pub readers: u32,
    pub writers: u32,
    pub is_active: bool,
}

impl Pipe {
    pub fn new(read_fd: i32, write_fd: i32) -> Self {
        Self {
            read_fd,
            write_fd,
            buffer: Vec::new(),
            readers: 1,
            writers: 1,
            is_active: true,
        }
    }
    
    pub fn write(&mut self, data: &[u8]) -> Result<usize, &'static str> {
        if !self.is_active || self.writers == 0 {
            return Err("Broken pipe");
        }
        
        let available_space = PIPE_BUFFER_SIZE - self.buffer.len();
        let bytes_to_write = core::cmp::min(data.len(), available_space);
        
        for i in 0..bytes_to_write {
            if self.buffer.push(data[i]).is_err() {
                break;
            }
        }
        
        UART.write_str("Pipe write: ");
        UART.put_hex(bytes_to_write as u32);
        UART.write_str(" bytes\n");
        
        Ok(bytes_to_write)
    }
    
    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize, &'static str> {
        if !self.is_active {
            return Err("Pipe closed");
        }
        
        let bytes_to_read = core::cmp::min(buf.len(), self.buffer.len());
        
        for i in 0..bytes_to_read {
            buf[i] = self.buffer.remove(0);
        }
        
        UART.write_str("Pipe read: ");
        UART.put_hex(bytes_to_read as u32);
        UART.write_str(" bytes\n");
        
        if bytes_to_read == 0 && self.writers == 0 {
            // EOF - no more writers
            Ok(0)
        } else {
            Ok(bytes_to_read)
        }
    }
    
    pub fn close_read_end(&mut self) {
        if self.readers > 0 {
            self.readers -= 1;
        }
        if self.readers == 0 && self.writers == 0 {
            self.is_active = false;
        }
    }
    
    pub fn close_write_end(&mut self) {
        if self.writers > 0 {
            self.writers -= 1;
        }
        if self.readers == 0 && self.writers == 0 {
            self.is_active = false;
        }
    }
    
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }
    
    pub fn is_full(&self) -> bool {
        self.buffer.len() >= PIPE_BUFFER_SIZE
    }
}

// Message Queue implementation
#[derive(Debug, Clone)]
pub struct Message {
    pub msg_type: i32,
    pub size: usize,
    pub data: Vec<u8, MAX_MESSAGE_SIZE>,
}

impl Message {
    pub fn new(msg_type: i32, data: &[u8]) -> Result<Self, &'static str> {
        if data.len() > MAX_MESSAGE_SIZE {
            return Err("Message too large");
        }
        
        let mut message_data = Vec::new();
        for &byte in data {
            let _ = message_data.push(byte);
        }
        
        Ok(Self {
            msg_type,
            size: data.len(),
            data: message_data,
        })
    }
}

#[derive(Debug)]
pub struct MessageQueue {
    pub id: i32,
    pub messages: Vec<Message, MAX_MESSAGES_PER_QUEUE>,
    pub max_size: usize,
    pub permissions: u32,
    pub created_by: u32, // PID of creator
}

impl MessageQueue {
    pub fn new(id: i32, permissions: u32, creator_pid: u32) -> Self {
        Self {
            id,
            messages: Vec::new(),
            max_size: MAX_MESSAGES_PER_QUEUE,
            permissions,
            created_by: creator_pid,
        }
    }
    
    pub fn send_message(&mut self, message: Message) -> Result<(), &'static str> {
        if self.messages.is_full() {
            return Err("Message queue full");
        }
        
        let _ = self.messages.push(message);
        UART.write_str("Message sent to queue ");
        UART.put_hex(self.id as u32);
        UART.write_str("\n");
        
        Ok(())
    }
    
    pub fn receive_message(&mut self, msg_type: i32) -> Option<Message> {
        // Find message with matching type (0 means any type)
        let mut index = None;
        for (i, msg) in self.messages.iter().enumerate() {
            if msg_type == 0 || msg.msg_type == msg_type {
                index = Some(i);
                break;
            }
        }
        
        if let Some(i) = index {
            let message = self.messages.remove(i);
            UART.write_str("Message received from queue ");
            UART.put_hex(self.id as u32);
            UART.write_str("\n");
            Some(message)
        } else {
            None
        }
    }
    
    pub fn message_count(&self) -> usize {
        self.messages.len()
    }
}

// Shared Memory Segment
#[derive(Debug)]
pub struct SharedMemorySegment {
    pub id: i32,
    pub size: usize,
    pub data: Vec<u8, 4096>, // Simplified fixed size
    pub permissions: u32,
    pub attached_processes: Vec<u32, 32>, // PIDs of attached processes
    pub created_by: u32,
}

impl SharedMemorySegment {
    pub fn new(id: i32, size: usize, permissions: u32, creator_pid: u32) -> Self {
        Self {
            id,
            size: core::cmp::min(size, 4096), // Cap at 4KB for simplicity
            data: Vec::new(),
            permissions,
            attached_processes: Vec::new(),
            created_by: creator_pid,
        }
    }
    
    pub fn attach_process(&mut self, pid: u32) -> Result<(), &'static str> {
        if self.attached_processes.is_full() {
            return Err("Too many attached processes");
        }
        
        for &attached_pid in &self.attached_processes {
            if attached_pid == pid {
                return Err("Process already attached");
            }
        }
        
        let _ = self.attached_processes.push(pid);
        UART.write_str("Process ");
        UART.put_hex(pid);
        UART.write_str(" attached to shared memory ");
        UART.put_hex(self.id as u32);
        UART.write_str("\n");
        
        Ok(())
    }
    
    pub fn detach_process(&mut self, pid: u32) -> Result<(), &'static str> {
        for (i, &attached_pid) in self.attached_processes.iter().enumerate() {
            if attached_pid == pid {
                self.attached_processes.remove(i);
                UART.write_str("Process ");
                UART.put_hex(pid);
                UART.write_str(" detached from shared memory ");
                UART.put_hex(self.id as u32);
                UART.write_str("\n");
                return Ok(());
            }
        }
        
        Err("Process not attached")
    }
    
    pub fn write_data(&mut self, offset: usize, data: &[u8]) -> Result<usize, &'static str> {
        if offset >= self.size {
            return Err("Offset out of bounds");
        }
        
        let available_space = self.size - offset;
        let bytes_to_write = core::cmp::min(data.len(), available_space);
        
        // Ensure data vector is large enough
        while self.data.len() < offset + bytes_to_write {
            if self.data.push(0).is_err() {
                break;
            }
        }
        
        for i in 0..bytes_to_write {
            if offset + i < self.data.len() {
                self.data[offset + i] = data[i];
            }
        }
        
        Ok(bytes_to_write)
    }
    
    pub fn read_data(&self, offset: usize, buf: &mut [u8]) -> Result<usize, &'static str> {
        if offset >= self.size {
            return Err("Offset out of bounds");
        }
        
        let available_data = core::cmp::min(self.data.len().saturating_sub(offset), self.size - offset);
        let bytes_to_read = core::cmp::min(buf.len(), available_data);
        
        for i in 0..bytes_to_read {
            if offset + i < self.data.len() {
                buf[i] = self.data[offset + i];
            } else {
                buf[i] = 0;
            }
        }
        
        Ok(bytes_to_read)
    }
}

// IPC Manager
pub struct IPCManager {
    pipes: Vec<Pipe, MAX_PIPES>,
    message_queues: Vec<MessageQueue, MAX_MESSAGE_QUEUES>,
    shared_memory: Vec<SharedMemorySegment, 16>,
    next_pipe_id: i32,
    next_msgq_id: i32,
    next_shm_id: i32,
}

impl IPCManager {
    pub fn new() -> Self {
        Self {
            pipes: Vec::new(),
            message_queues: Vec::new(),
            shared_memory: Vec::new(),
            next_pipe_id: 100,
            next_msgq_id: 1000,
            next_shm_id: 10000,
        }
    }
    
    pub fn create_pipe(&mut self) -> Result<(i32, i32), &'static str> {
        if self.pipes.is_full() {
            return Err("Too many pipes");
        }
        
        let read_fd = self.next_pipe_id;
        let write_fd = self.next_pipe_id + 1;
        self.next_pipe_id += 2;
        
        let pipe = Pipe::new(read_fd, write_fd);
        let _ = self.pipes.push(pipe);
        
        UART.write_str("Created pipe: read_fd=");
        UART.put_hex(read_fd as u32);
        UART.write_str(", write_fd=");
        UART.put_hex(write_fd as u32);
        UART.write_str("\n");
        
        Ok((read_fd, write_fd))
    }
    
    pub fn get_pipe_mut(&mut self, fd: i32) -> Option<&mut Pipe> {
        self.pipes.iter_mut().find(|p| p.read_fd == fd || p.write_fd == fd)
    }
    
    pub fn close_pipe(&mut self, fd: i32) -> Result<(), &'static str> {
        if let Some(pipe) = self.get_pipe_mut(fd) {
            if pipe.read_fd == fd {
                pipe.close_read_end();
            } else if pipe.write_fd == fd {
                pipe.close_write_end();
            }
            
            UART.write_str("Closed pipe fd ");
            UART.put_hex(fd as u32);
            UART.write_str("\n");
            
            Ok(())
        } else {
            Err("Pipe not found")
        }
    }
    
    pub fn create_message_queue(&mut self, key: i32, permissions: u32, creator_pid: u32) -> Result<i32, &'static str> {
        if self.message_queues.is_full() {
            return Err("Too many message queues");
        }
        
        let id = self.next_msgq_id;
        self.next_msgq_id += 1;
        
        let msgq = MessageQueue::new(id, permissions, creator_pid);
        let _ = self.message_queues.push(msgq);
        
        UART.write_str("Created message queue ");
        UART.put_hex(id as u32);
        UART.write_str(" with key ");
        UART.put_hex(key as u32);
        UART.write_str("\n");
        
        Ok(id)
    }
    
    pub fn get_message_queue_mut(&mut self, id: i32) -> Option<&mut MessageQueue> {
        self.message_queues.iter_mut().find(|q| q.id == id)
    }
    
    pub fn create_shared_memory(&mut self, key: i32, size: usize, permissions: u32, creator_pid: u32) -> Result<i32, &'static str> {
        if self.shared_memory.is_full() {
            return Err("Too many shared memory segments");
        }
        
        let id = self.next_shm_id;
        self.next_shm_id += 1;
        
        let shm = SharedMemorySegment::new(id, size, permissions, creator_pid);
        let _ = self.shared_memory.push(shm);
        
        UART.write_str("Created shared memory ");
        UART.put_hex(id as u32);
        UART.write_str(" with key ");
        UART.put_hex(key as u32);
        UART.write_str(" size ");
        UART.put_hex(size as u32);
        UART.write_str("\n");
        
        Ok(id)
    }
    
    pub fn get_shared_memory_mut(&mut self, id: i32) -> Option<&mut SharedMemorySegment> {
        self.shared_memory.iter_mut().find(|s| s.id == id)
    }
    
    pub fn cleanup_process_ipc(&mut self, pid: u32) {
        // Close pipes associated with process
        for pipe in &mut self.pipes {
            if pipe.readers > 0 || pipe.writers > 0 {
                // This is simplified - in real implementation we'd track which process owns which end
                pipe.close_read_end();
                pipe.close_write_end();
            }
        }
        
        // Detach from shared memory
        for shm in &mut self.shared_memory {
            let _ = shm.detach_process(pid);
        }
        
        UART.write_str("Cleaned up IPC for process ");
        UART.put_hex(pid);
        UART.write_str("\n");
    }
    
    pub fn get_stats(&self) -> (usize, usize, usize) {
        (self.pipes.len(), self.message_queues.len(), self.shared_memory.len())
    }
}

// Global IPC manager
static mut GLOBAL_IPC_MANAGER: IPCManager = IPCManager {
    pipes: Vec::new(),
    message_queues: Vec::new(),
    shared_memory: Vec::new(),
    next_pipe_id: 100,
    next_msgq_id: 1000,
    next_shm_id: 10000,
};

pub fn init_ipc() {
    unsafe {
        GLOBAL_IPC_MANAGER = IPCManager::new();
    }
    UART.write_str("IPC system initialized\n");
}

pub fn create_pipe() -> Result<(i32, i32), &'static str> {
    unsafe { GLOBAL_IPC_MANAGER.create_pipe() }
}

pub fn pipe_write(fd: i32, data: &[u8]) -> Result<usize, &'static str> {
    unsafe {
        if let Some(pipe) = GLOBAL_IPC_MANAGER.get_pipe_mut(fd) {
            if pipe.write_fd == fd {
                pipe.write(data)
            } else {
                Err("Not a write file descriptor")
            }
        } else {
            Err("Pipe not found")
        }
    }
}

pub fn pipe_read(fd: i32, buf: &mut [u8]) -> Result<usize, &'static str> {
    unsafe {
        if let Some(pipe) = GLOBAL_IPC_MANAGER.get_pipe_mut(fd) {
            if pipe.read_fd == fd {
                pipe.read(buf)
            } else {
                Err("Not a read file descriptor")
            }
        } else {
            Err("Pipe not found")
        }
    }
}

pub fn close_pipe(fd: i32) -> Result<(), &'static str> {
    unsafe { GLOBAL_IPC_MANAGER.close_pipe(fd) }
}

pub fn create_message_queue(key: i32, permissions: u32, creator_pid: u32) -> Result<i32, &'static str> {
    unsafe { GLOBAL_IPC_MANAGER.create_message_queue(key, permissions, creator_pid) }
}

pub fn send_message(msgq_id: i32, msg_type: i32, data: &[u8]) -> Result<(), &'static str> {
    let message = Message::new(msg_type, data)?;
    unsafe {
        if let Some(msgq) = GLOBAL_IPC_MANAGER.get_message_queue_mut(msgq_id) {
            msgq.send_message(message)
        } else {
            Err("Message queue not found")
        }
    }
}

pub fn receive_message(msgq_id: i32, msg_type: i32) -> Option<Message> {
    unsafe {
        if let Some(msgq) = GLOBAL_IPC_MANAGER.get_message_queue_mut(msgq_id) {
            msgq.receive_message(msg_type)
        } else {
            None
        }
    }
}

pub fn create_shared_memory(key: i32, size: usize, permissions: u32, creator_pid: u32) -> Result<i32, &'static str> {
    unsafe { GLOBAL_IPC_MANAGER.create_shared_memory(key, size, permissions, creator_pid) }
}

pub fn attach_shared_memory(shm_id: i32, pid: u32) -> Result<(), &'static str> {
    unsafe {
        if let Some(shm) = GLOBAL_IPC_MANAGER.get_shared_memory_mut(shm_id) {
            shm.attach_process(pid)
        } else {
            Err("Shared memory not found")
        }
    }
}

pub fn detach_shared_memory(shm_id: i32, pid: u32) -> Result<(), &'static str> {
    unsafe {
        if let Some(shm) = GLOBAL_IPC_MANAGER.get_shared_memory_mut(shm_id) {
            shm.detach_process(pid)
        } else {
            Err("Shared memory not found")
        }
    }
}

pub fn cleanup_process_ipc(pid: u32) {
    unsafe {
        GLOBAL_IPC_MANAGER.cleanup_process_ipc(pid);
    }
}

pub fn get_ipc_stats() -> (usize, usize, usize) {
    unsafe { GLOBAL_IPC_MANAGER.get_stats() }
}
