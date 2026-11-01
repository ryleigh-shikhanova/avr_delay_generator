use crate::{DelayConfig, RoundingMode};
use crate::error::DelayError;

/// Converts a requested delay into required num of CPU cycles
///
/// Uses the CPU clock frequency in Hz and requested delay in ns
/// 
/// Formula:
/// - C = (clock_hz * delay_ns) / 1_000_000_000
/// - Up: rounds toward positive infinity (ceiling)
/// - Nearest: rounds to the closest integer (ties round up)
/// 
///  Uses u128 arithemtic to prevent overflow during multiplication
/// 
/// Returns:
/// - Ok(cycles): The calculated num of CPU cycles
/// - Err(error): The result exceeds max value of u64
pub fn delay_to_cycles(config: &DelayConfig) -> Result<u64, std::num::TryFromIntError> {
    let clock = config.clock_hz as u128;
    let delay = config.delay_ns as u128;

    let numerator = clock * delay;
    let denominator = 1_000_000_000_u128;

    let cycles = match config.rounding_mode {
        RoundingMode::Up => numerator.div_ceil(denominator),
        RoundingMode::Nearest => (numerator + denominator / 2) / denominator,
    };

    u64::try_from(cycles)
}

/// Calculates the total cycles consumed by an 8-bit AVR delay loop.
///
/// Assumes an LDI initialization followed by DEC and BRNE.
/// Valid for iteration counts from 1 to 256.
///
/// Timing:
/// - LDI: 1 cycle
/// - DEC: 1 cycle per iteration
/// - BRNE: 2 cycles when taken, 1 when not taken
///
/// C(n) = 1 + n + 2(n - 1) + 1 = 3n
pub fn loop8_cycles(iterations: u16) -> Result<u64, DelayError> {
    if !(1..=256).contains(&iterations) {
        return Err(DelayError::InvalidIterations {
            requested: u32::from(iterations),
            maximum: 256,
        });
    }

    Ok(u64::from(iterations) * 3)
}

/// Calculates the total cycles consumed by an 16-bit AVR delay loop.
///
/// The assumed initialization is:
/// - ldi register, lo8(n)
/// - ldi register+1, hi8(n)
/// - Followed by SBIW and BRNE
/// 
/// Valid for iteration counts from 1 to 65,536.
///
/// Timing:
/// - LDI: 1 cycle * 2 = 2 total cycles
/// - SBIW: 2 cycles per iteration --> can only operate on r25:r24, r27:r26, r29:r28, r31:r30
/// - BRNE: 2 cycles when taken, 1 when not taken
///
/// C(n) = 2 + 2n + 2(n - 1) + 1 = 4n + 1
pub fn loop16_cycles(iterations: u32) -> Result<u64, DelayError> {
    if !(1..=65_536).contains(&iterations) {
        return Err(DelayError::InvalidIterations {
            requested: iterations,
            maximum: 65_536,
        });
    }
    
    Ok(u64::from(iterations) * 4 + 1)
} 

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loop8_cycles() {
    assert_eq!(loop8_cycles(1).unwrap(), 3);
    assert_eq!(loop8_cycles(100).unwrap(), 300);
    assert_eq!(loop8_cycles(256).unwrap(), 768);
    }

    #[test]
    fn test_loop16_cycles() {
        assert_eq!(loop16_cycles(1000).unwrap(), 4001);
    }

    #[test]
    fn test_loop16_invalid_iterations() {
        assert!(loop16_cycles(0).is_err());
        assert!(loop16_cycles(65_537).is_err());
    }
}