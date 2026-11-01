pub mod error;
pub mod generator;
pub mod instruction;
pub mod planner;
pub mod timing;

#[derive(Debug, Clone, Copy)]
pub enum GenerationMode {
    Inline,
    Subroutine,
}

pub enum RoundingMode {
    Up,
    Nearest,
}

pub struct DelayConfig {
    pub clock_hz: u64,
    pub delay_ns: u64,
    pub rounding_mode: RoundingMode,
}
