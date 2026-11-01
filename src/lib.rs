pub mod error;
pub mod instruction;
pub mod timing;
pub mod planner;
pub mod generator;

pub enum RoundingMode {
    Up,
    Nearest,
}

pub struct DelayConfig {
    pub clock_hz: u64,
    pub delay_ns: u64,
    pub rounding_mode: RoundingMode,
}

