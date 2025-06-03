// Process Management for UNIX-like OS
// Basic process scheduling and management

use heapless::Vec;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ProcessState {
    Ready,
    Running,
    Sleeping,
    Terminated,
}

#[derive(Clone, Copy)]
pub struct Process {
    pub pid: u32,
    pub ppid: u32,           // Parent process ID
    pub state: ProcessState,
    pub stack_ptr: u64,      // Stack pointer
    pub entry_point: u64,    // Program entry point
    pub priority: u8,        // Process priority (0-255)
    pub time_slice: u32,     // Time slice in ms
    pub used_time: u32,      // Used CPU time
}

const MAX_PROCESSES: usize = 64;
const DEFAULT_TIME_SLICE: u32 = 10; // 10ms

pub struct ProcessManager {
    processes: Vec<Process, MAX_PROCESSES>,
    current_pid: u32,
    next_pid: u32,
    scheduler_tick: u32,
}

impl ProcessManager {
    pub const fn new() -> Self {
        Self {
            processes: Vec::new(),
            current_pid: 0,
            next_pid: 1,
            scheduler_tick: 0,
        }
    }
    
    /// 新しいプロセスを作成
    pub fn create_process(&mut self, entry_point: u64, parent_pid: u32) -> Option<u32> {
        if self.processes.is_full() {
            return None;
        }
        
        let pid = self.next_pid;
        self.next_pid += 1;
        
        let process = Process {
            pid,
            ppid: parent_pid,
            state: ProcessState::Ready,
            stack_ptr: 0x400000 + (pid as u64 * 0x100000), // 1MB stack per process
            entry_point,
            priority: 128, // Default priority
            time_slice: DEFAULT_TIME_SLICE,
            used_time: 0,
        };
        
        let _ = self.processes.push(process);
        Some(pid)
    }
    
    /// initプロセス（PID 1）を作成
    pub fn create_init_process(&mut self, entry_point: u64) -> Option<u32> {
        self.next_pid = 1;
        self.create_process(entry_point, 0)
    }
    
    /// 現在のプロセスIDを取得
    pub fn current_pid(&self) -> u32 {
        self.current_pid
    }
    
    /// プロセス状態を変更
    pub fn set_process_state(&mut self, pid: u32, state: ProcessState) -> bool {
        for process in &mut self.processes {
            if process.pid == pid {
                process.state = state;
                return true;
            }
        }
        false
    }
    
    /// ラウンドロビンスケジューリング
    pub fn schedule(&mut self) -> Option<u32> {
        self.scheduler_tick += 1;
        
        // 現在のプロセスの時間を更新
        if let Some(current) = self.get_process_mut(self.current_pid) {
            current.used_time += 1;
            
            // タイムスライス終了またはプロセス終了
            if current.used_time >= current.time_slice || current.state != ProcessState::Running {
                current.state = if current.state == ProcessState::Running {
                    ProcessState::Ready
                } else {
                    current.state
                };
                current.used_time = 0;
            }
        }
        
        // 次に実行するプロセスを選択
        let current_index = self.processes.iter()
            .position(|p| p.pid == self.current_pid)
            .unwrap_or(0);
            
        for i in 1..=self.processes.len() {
            let index = (current_index + i) % self.processes.len();
            if self.processes[index].state == ProcessState::Ready {
                self.current_pid = self.processes[index].pid;
                self.processes[index].state = ProcessState::Running;
                return Some(self.current_pid);
            }
        }
        
        None
    }
    
    /// プロセス情報を取得
    pub fn get_process(&self, pid: u32) -> Option<&Process> {
        self.processes.iter().find(|p| p.pid == pid)
    }
    
    fn get_process_mut(&mut self, pid: u32) -> Option<&mut Process> {
        self.processes.iter_mut().find(|p| p.pid == pid)
    }
    
    /// 全プロセス一覧を取得
    pub fn list_processes(&self) -> &[Process] {
        &self.processes
    }
    
    /// プロセス終了
    pub fn terminate_process(&mut self, pid: u32) -> bool {
        if let Some(process) = self.get_process_mut(pid) {
            process.state = ProcessState::Terminated;
            
            // 子プロセスの親をinitプロセス(PID 1)に変更
            for p in &mut self.processes {
                if p.ppid == pid {
                    p.ppid = 1;
                }
            }
            
            true
        } else {
            false
        }
    }
}

// グローバルプロセスマネージャー
pub static mut PROCESS_MANAGER: ProcessManager = ProcessManager::new();
