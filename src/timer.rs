// System Timer for Raspberry Pi 5
// Provides timing services for the OS

const SYSTEM_TIMER_BASE: u64 = 0xfe003000;

// Timer registers
const TIMER_CS: u32 = 0x00;   // Control/Status
const TIMER_CLO: u32 = 0x04;  // Counter Lower 32 bits
const TIMER_CHI: u32 = 0x08;  // Counter Higher 32 bits
const TIMER_C0: u32 = 0x0C;   // Compare 0
const TIMER_C1: u32 = 0x10;   // Compare 1
const TIMER_C2: u32 = 0x14;   // Compare 2
const TIMER_C3: u32 = 0x18;   // Compare 3

pub struct Timer {
    base_addr: u64,
}

impl Timer {
    pub const fn new() -> Self {
        Self {
            base_addr: SYSTEM_TIMER_BASE,
        }
    }
    
    /// システムタイマー初期化
    pub fn init(&self) {
        // BCM2712のシステムタイマーは1MHz
        // 基本的な初期化のみ実行
    }
    
    /// 現在の時刻をマイクロ秒で取得
    pub fn get_time_us(&self) -> u64 {
        unsafe {
            let timer_base = self.base_addr as *const u32;
            let lo = core::ptr::read_volatile(timer_base.add((TIMER_CLO / 4) as usize));
            let hi = core::ptr::read_volatile(timer_base.add((TIMER_CHI / 4) as usize));
            ((hi as u64) << 32) | (lo as u64)
        }
    }
    
    /// 指定時間待機（マイクロ秒）
    pub fn delay_us(&self, us: u32) {
        let start = self.get_time_us();
        while self.get_time_us() - start < us as u64 {
            core::hint::spin_loop();
        }
    }
    
    /// 指定時間待機（ミリ秒）
    pub fn delay_ms(&self, ms: u32) {
        self.delay_us(ms * 1000);
    }
    
    /// システム起動からの時間を秒で取得
    pub fn get_uptime_seconds(&self) -> u32 {
        (self.get_time_us() / 1_000_000) as u32
    }
}

// グローバルタイマーインスタンス
pub static TIMER: Timer = Timer::new();

// グローバル関数（他のモジュールから使用可能）
pub fn delay_us(us: u32) {
    TIMER.delay_us(us);
}

pub fn delay_ms(ms: u32) {
    TIMER.delay_ms(ms);
}

pub fn get_time_us() -> u64 {
    TIMER.get_time_us()
}

pub fn get_uptime_seconds() -> u32 {
    TIMER.get_uptime_seconds()
}
