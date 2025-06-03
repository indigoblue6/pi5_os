// Signal System for UNIX Compatibility
// POSIX signal handling implementation

use crate::process::{PROCESS_MANAGER, ProcessState};
use crate::uart::UART;
use heapless::Vec;

// POSIX signals
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Signal {
    SIGHUP = 1,
    SIGINT = 2,
    SIGQUIT = 3,
    SIGILL = 4,
    SIGTRAP = 5,
    SIGABRT = 6,
    SIGBUS = 7,
    SIGFPE = 8,
    SIGKILL = 9,
    SIGUSR1 = 10,
    SIGSEGV = 11,
    SIGUSR2 = 12,
    SIGPIPE = 13,
    SIGALRM = 14,
    SIGTERM = 15,
    SIGSTKFLT = 16,
    SIGCHLD = 17,
    SIGCONT = 18,
    SIGSTOP = 19,
    SIGTSTP = 20,
    SIGTTIN = 21,
    SIGTTOU = 22,
    SIGURG = 23,
    SIGXCPU = 24,
    SIGXFSZ = 25,
    SIGVTALRM = 26,
    SIGPROF = 27,
    SIGWINCH = 28,
    SIGIO = 29,
    SIGPWR = 30,
    SIGSYS = 31,
}

impl Signal {
    pub fn from_i32(value: i32) -> Option<Self> {
        match value {
            1 => Some(Signal::SIGHUP),
            2 => Some(Signal::SIGINT),
            3 => Some(Signal::SIGQUIT),
            4 => Some(Signal::SIGILL),
            5 => Some(Signal::SIGTRAP),
            6 => Some(Signal::SIGABRT),
            7 => Some(Signal::SIGBUS),
            8 => Some(Signal::SIGFPE),
            9 => Some(Signal::SIGKILL),
            10 => Some(Signal::SIGUSR1),
            11 => Some(Signal::SIGSEGV),
            12 => Some(Signal::SIGUSR2),
            13 => Some(Signal::SIGPIPE),
            14 => Some(Signal::SIGALRM),
            15 => Some(Signal::SIGTERM),
            16 => Some(Signal::SIGSTKFLT),
            17 => Some(Signal::SIGCHLD),
            18 => Some(Signal::SIGCONT),
            19 => Some(Signal::SIGSTOP),
            20 => Some(Signal::SIGTSTP),
            21 => Some(Signal::SIGTTIN),
            22 => Some(Signal::SIGTTOU),
            23 => Some(Signal::SIGURG),
            24 => Some(Signal::SIGXCPU),
            25 => Some(Signal::SIGXFSZ),
            26 => Some(Signal::SIGVTALRM),
            27 => Some(Signal::SIGPROF),
            28 => Some(Signal::SIGWINCH),
            29 => Some(Signal::SIGIO),
            30 => Some(Signal::SIGPWR),
            31 => Some(Signal::SIGSYS),
            _ => None,
        }
    }
    
    pub fn name(&self) -> &'static str {
        match self {
            Signal::SIGHUP => "SIGHUP",
            Signal::SIGINT => "SIGINT",
            Signal::SIGQUIT => "SIGQUIT",
            Signal::SIGILL => "SIGILL",
            Signal::SIGTRAP => "SIGTRAP",
            Signal::SIGABRT => "SIGABRT",
            Signal::SIGBUS => "SIGBUS",
            Signal::SIGFPE => "SIGFPE",
            Signal::SIGKILL => "SIGKILL",
            Signal::SIGUSR1 => "SIGUSR1",
            Signal::SIGSEGV => "SIGSEGV",
            Signal::SIGUSR2 => "SIGUSR2",
            Signal::SIGPIPE => "SIGPIPE",
            Signal::SIGALRM => "SIGALRM",
            Signal::SIGTERM => "SIGTERM",
            Signal::SIGSTKFLT => "SIGSTKFLT",
            Signal::SIGCHLD => "SIGCHLD",
            Signal::SIGCONT => "SIGCONT",
            Signal::SIGSTOP => "SIGSTOP",
            Signal::SIGTSTP => "SIGTSTP",
            Signal::SIGTTIN => "SIGTTIN",
            Signal::SIGTTOU => "SIGTTOU",
            Signal::SIGURG => "SIGURG",
            Signal::SIGXCPU => "SIGXCPU",
            Signal::SIGXFSZ => "SIGXFSZ",
            Signal::SIGVTALRM => "SIGVTALRM",
            Signal::SIGPROF => "SIGPROF",
            Signal::SIGWINCH => "SIGWINCH",
            Signal::SIGIO => "SIGIO",
            Signal::SIGPWR => "SIGPWR",
            Signal::SIGSYS => "SIGSYS",
        }
    }
    
    pub fn is_uncatchable(&self) -> bool {
        matches!(self, Signal::SIGKILL | Signal::SIGSTOP)
    }
    
    pub fn default_action(&self) -> SignalAction {
        match self {
            Signal::SIGCHLD | Signal::SIGURG | Signal::SIGWINCH => SignalAction::Ignore,
            Signal::SIGSTOP | Signal::SIGTSTP | Signal::SIGTTIN | Signal::SIGTTOU => SignalAction::Stop,
            Signal::SIGCONT => SignalAction::Continue,
            _ => SignalAction::Terminate,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SignalAction {
    Default,
    Ignore,
    Terminate,
    Stop,
    Continue,
    Core,       // Terminate with core dump
    Custom(u64), // Custom handler address
}

#[derive(Clone, Copy, Debug)]
pub struct PendingSignal {
    pub signal: Signal,
    pub sender_pid: u32,
}

const MAX_PENDING_SIGNALS: usize = 32;

pub struct SignalHandler {
    signal_mask: u64,  // Blocked signals bitmap
    pending_signals: Vec<PendingSignal, MAX_PENDING_SIGNALS>,
    signal_handlers: [SignalAction; 32], // One handler per signal
}

impl SignalHandler {
    pub fn new() -> Self {
        let mut handlers = [SignalAction::Default; 32];
        
        // Set default actions for specific signals
        for i in 1..=31 {
            if let Some(signal) = Signal::from_i32(i) {
                handlers[(i - 1) as usize] = signal.default_action();
            }
        }
        
        Self {
            signal_mask: 0,
            pending_signals: Vec::new(),
            signal_handlers: handlers,
        }
    }
    
    pub fn send_signal(&mut self, target_pid: u32, signal: Signal, sender_pid: u32) -> Result<(), &'static str> {
        UART.write_str("Sending signal ");
        UART.write_str(signal.name());
        UART.write_str(" to PID ");
        UART.put_hex(target_pid);
        UART.write_str(" from PID ");
        UART.put_hex(sender_pid);
        UART.write_str("\n");
        
        // Check if signal is blocked
        let signal_bit = 1u64 << (signal as i32 - 1);
        if self.signal_mask & signal_bit != 0 && !signal.is_uncatchable() {
            // Signal is blocked, add to pending
            if !self.pending_signals.is_full() {
                let _ = self.pending_signals.push(PendingSignal {
                    signal,
                    sender_pid,
                });
                UART.write_str("Signal blocked, added to pending\n");
                return Ok(());
            } else {
                return Err("Too many pending signals");
            }
        }
        
        // Deliver signal immediately
        self.deliver_signal(target_pid, signal, sender_pid)
    }
    
    fn deliver_signal(&mut self, target_pid: u32, signal: Signal, _sender_pid: u32) -> Result<(), &'static str> {
        let handler_index = (signal as i32 - 1) as usize;
        let action = self.signal_handlers[handler_index];
        
        UART.write_str("Delivering signal ");
        UART.write_str(signal.name());
        UART.write_str(" with action: ");
        
        match action {
            SignalAction::Default => {
                UART.write_str("DEFAULT\n");
                self.default_signal_action(target_pid, signal)
            }
            SignalAction::Ignore => {
                UART.write_str("IGNORE\n");
                Ok(())
            }
            SignalAction::Terminate => {
                UART.write_str("TERMINATE\n");
                self.terminate_process(target_pid)
            }
            SignalAction::Stop => {
                UART.write_str("STOP\n");
                self.stop_process(target_pid)
            }
            SignalAction::Continue => {
                UART.write_str("CONTINUE\n");
                self.continue_process(target_pid)
            }
            SignalAction::Core => {
                UART.write_str("CORE_DUMP\n");
                self.core_dump_process(target_pid)
            }
            SignalAction::Custom(handler_addr) => {
                UART.write_str("CUSTOM at 0x");
                UART.put_hex(handler_addr as u32);
                UART.write_str("\n");
                self.call_custom_handler(target_pid, signal, handler_addr)
            }
        }
    }
    
    fn default_signal_action(&mut self, target_pid: u32, signal: Signal) -> Result<(), &'static str> {
        match signal.default_action() {
            SignalAction::Terminate => self.terminate_process(target_pid),
            SignalAction::Stop => self.stop_process(target_pid),
            SignalAction::Continue => self.continue_process(target_pid),
            SignalAction::Ignore => Ok(()),
            _ => self.terminate_process(target_pid),
        }
    }
    
    fn terminate_process(&mut self, target_pid: u32) -> Result<(), &'static str> {
        unsafe {
            if PROCESS_MANAGER.terminate_process(target_pid) {
                UART.write_str("Process ");
                UART.put_hex(target_pid);
                UART.write_str(" terminated by signal\n");
                Ok(())
            } else {
                Err("Failed to terminate process")
            }
        }
    }
    
    fn stop_process(&mut self, target_pid: u32) -> Result<(), &'static str> {
        unsafe {
            if PROCESS_MANAGER.set_process_state(target_pid, ProcessState::Sleeping) {
                UART.write_str("Process ");
                UART.put_hex(target_pid);
                UART.write_str(" stopped by signal\n");
                Ok(())
            } else {
                Err("Failed to stop process")
            }
        }
    }
    
    fn continue_process(&mut self, target_pid: u32) -> Result<(), &'static str> {
        unsafe {
            if PROCESS_MANAGER.set_process_state(target_pid, ProcessState::Ready) {
                UART.write_str("Process ");
                UART.put_hex(target_pid);
                UART.write_str(" continued by signal\n");
                Ok(())
            } else {
                Err("Failed to continue process")
            }
        }
    }
    
    fn core_dump_process(&mut self, target_pid: u32) -> Result<(), &'static str> {
        UART.write_str("Core dump for PID ");
        UART.put_hex(target_pid);
        UART.write_str(" (simplified)\n");
        
        // In a real implementation, this would dump process memory
        unsafe {
            if let Some(process) = PROCESS_MANAGER.get_process(target_pid) {
                UART.write_str("PID: ");
                UART.put_hex(process.pid);
                UART.write_str("\n");
                UART.write_str("PPID: ");
                UART.put_hex(process.ppid);
                UART.write_str("\n");
                UART.write_str("Entry Point: 0x");
                UART.put_hex(process.entry_point as u32);
                UART.write_str("\n");
                UART.write_str("Stack Pointer: 0x");
                UART.put_hex(process.stack_ptr as u32);
                UART.write_str("\n");
            }
        }
        
        self.terminate_process(target_pid)
    }
    
    fn call_custom_handler(&mut self, _target_pid: u32, signal: Signal, _handler_addr: u64) -> Result<(), &'static str> {
        UART.write_str("Custom signal handler for ");
        UART.write_str(signal.name());
        UART.write_str(" not fully implemented\n");
        // In a real implementation, this would set up a signal stack frame
        // and jump to the custom handler
        Ok(())
    }
    
    pub fn set_signal_handler(&mut self, signal: Signal, action: SignalAction) -> Result<(), &'static str> {
        if signal.is_uncatchable() {
            return Err("Cannot catch SIGKILL or SIGSTOP");
        }
        
        let handler_index = (signal as i32 - 1) as usize;
        self.signal_handlers[handler_index] = action;
        
        UART.write_str("Signal handler set for ");
        UART.write_str(signal.name());
        UART.write_str("\n");
        
        Ok(())
    }
    
    pub fn block_signal(&mut self, signal: Signal) {
        if !signal.is_uncatchable() {
            let signal_bit = 1u64 << (signal as i32 - 1);
            self.signal_mask |= signal_bit;
            
            UART.write_str("Blocked signal ");
            UART.write_str(signal.name());
            UART.write_str("\n");
        }
    }
    
    pub fn unblock_signal(&mut self, signal: Signal) {
        let signal_bit = 1u64 << (signal as i32 - 1);
        self.signal_mask &= !signal_bit;
        
        UART.write_str("Unblocked signal ");
        UART.write_str(signal.name());
        UART.write_str("\n");
        
        // Check for pending signals to deliver
        self.check_pending_signals();
    }
    
    fn check_pending_signals(&mut self) {
        let mut i = 0;
        while i < self.pending_signals.len() {
            let pending = self.pending_signals[i];
            let signal_bit = 1u64 << (pending.signal as i32 - 1);
            
            if self.signal_mask & signal_bit == 0 {
                // Signal is no longer blocked, deliver it
                let _ = self.deliver_signal(0, pending.signal, pending.sender_pid); // PID 0 for current process
                self.pending_signals.remove(i);
            } else {
                i += 1;
            }
        }
    }
    
    pub fn handle_keyboard_interrupt(&mut self) {
        UART.write_str("Keyboard interrupt (Ctrl+C) detected\n");
        let current_pid = unsafe { PROCESS_MANAGER.current_pid() };
        let _ = self.send_signal(current_pid, Signal::SIGINT, 0);
    }
    
    pub fn get_signal_mask(&self) -> u64 {
        self.signal_mask
    }
    
    pub fn pending_signals_count(&self) -> usize {
        self.pending_signals.len()
    }
}

// Global signal handler
static mut GLOBAL_SIGNAL_HANDLER: SignalHandler = SignalHandler {
    signal_mask: 0,
    pending_signals: Vec::new(),
    signal_handlers: [SignalAction::Default; 32],
};

pub fn init_signals() {
    unsafe {
        GLOBAL_SIGNAL_HANDLER = SignalHandler::new();
    }
    UART.write_str("Signal system initialized\n");
}

pub fn send_signal(target_pid: u32, signal_num: i32, sender_pid: u32) -> Result<(), &'static str> {
    if let Some(signal) = Signal::from_i32(signal_num) {
        unsafe {
            GLOBAL_SIGNAL_HANDLER.send_signal(target_pid, signal, sender_pid)
        }
    } else {
        Err("Invalid signal number")
    }
}

pub fn set_signal_handler(signal_num: i32, action: SignalAction) -> Result<(), &'static str> {
    if let Some(signal) = Signal::from_i32(signal_num) {
        unsafe {
            GLOBAL_SIGNAL_HANDLER.set_signal_handler(signal, action)
        }
    } else {
        Err("Invalid signal number")
    }
}

pub fn block_signal(signal_num: i32) -> Result<(), &'static str> {
    if let Some(signal) = Signal::from_i32(signal_num) {
        unsafe {
            GLOBAL_SIGNAL_HANDLER.block_signal(signal);
        }
        Ok(())
    } else {
        Err("Invalid signal number")
    }
}

pub fn unblock_signal(signal_num: i32) -> Result<(), &'static str> {
    if let Some(signal) = Signal::from_i32(signal_num) {
        unsafe {
            GLOBAL_SIGNAL_HANDLER.unblock_signal(signal);
        }
        Ok(())
    } else {
        Err("Invalid signal number")
    }
}

pub fn handle_keyboard_interrupt() {
    unsafe {
        GLOBAL_SIGNAL_HANDLER.handle_keyboard_interrupt();
    }
}

pub fn get_signal_info() -> (u64, usize) {
    unsafe {
        (GLOBAL_SIGNAL_HANDLER.get_signal_mask(), GLOBAL_SIGNAL_HANDLER.pending_signals_count())
    }
}
