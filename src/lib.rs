pub enum RoundingMode {
    Up,
    Nearest,
}

pub struct DelayConfig {
    pub clock_hz: u64,
    pub delay_ns: u64,
    pub rounding_mode: RoundingMode,
}

pub fn calculate_cycles(config: &DelayConfig) -> u64 {
    let clock = config.clock_hz as u128;
    let delay = config.delay_ns as u128;

    let numerator = clock * delay;
    let denominator = 1_000_000_000_u128;

    let cycles = match config.rounding_mode {
        RoundingMode::Up => numerator.div_ceil(denominator),
        RoundingMode::Nearest => (numerator + denominator / 2) / denominator,
    };

    cycles as u64
}