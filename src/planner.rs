use crate::{error::DelayError};
use crate::timing::{loop8_cycles, loop16_cycles};

const BIT_8_OVERHEAD_CYCLES: u64 = 10;
const BIT_16_OVERHEAD_CYCLES: u64 = 15;

#[derive(Debug)]
pub enum LoopStrategy {
    Bit8,
    Bit16,
    PaddingOnly,
    Nested,
}
#[derive(Debug)]
pub struct DelayPlan {
    pub strategy: LoopStrategy,
    pub iterations: u32,
    pub remaining_cycles: u64,
    pub overhead_cycles: u64,
}

pub fn plan_delay(target_cycles: u64) -> Result<DelayPlan, DelayError> {
    let strategy = if target_cycles < 13 {
        LoopStrategy::PaddingOnly
    } else if target_cycles <= 778 {
        LoopStrategy::Bit8
    } else if target_cycles <= 262_161 {
        LoopStrategy::Bit16
    } else {
        LoopStrategy::Nested
    };

    let overhead_cycles = match strategy {
        LoopStrategy::Bit8 => BIT_8_OVERHEAD_CYCLES,
        LoopStrategy::Bit16 => BIT_16_OVERHEAD_CYCLES,
        LoopStrategy::PaddingOnly => 0,
        LoopStrategy::Nested => 0,
    };
    
    let number_of_iterations = match strategy {
        LoopStrategy::Bit8 => (target_cycles - BIT_8_OVERHEAD_CYCLES) / 3,
        LoopStrategy::Bit16 => (target_cycles - BIT_16_OVERHEAD_CYCLES - 1) / 4,
        LoopStrategy::PaddingOnly => 0,
        LoopStrategy::Nested => 0,
    };

    let loop_cycles = match strategy {
        LoopStrategy::Bit8 => {
            let iterations = u16::try_from(number_of_iterations).map_err(|_| DelayError::CycleOverflow)?;
            loop8_cycles(iterations)?
        }

        LoopStrategy::Bit16 => {
            let iterations = u32::try_from(number_of_iterations).map_err(|_| DelayError::CycleOverflow)?;
            loop16_cycles(iterations)?
        }

        LoopStrategy::PaddingOnly => 0,

        LoopStrategy::Nested => {
            return Err(DelayError::UnachievableDelay);
        }
    };

    let remaining_cycles = target_cycles.checked_sub(loop_cycles + overhead_cycles)
                                            .ok_or(DelayError::UnachievableDelay)?;
    let iterations = u32::try_from(number_of_iterations)
                            .map_err(|_| DelayError::CycleOverflow)?;

    Ok(DelayPlan {
        strategy,
        iterations,
        remaining_cycles,
        overhead_cycles,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_padding_only() {
        let plan = plan_delay(5).unwrap();

        assert!(matches!(plan.strategy, LoopStrategy::PaddingOnly));
        assert_eq!(plan.iterations, 0);
        assert_eq!(plan.remaining_cycles, 5);
    }

    #[test]
    fn test_bit8_minimum() {
        let plan = plan_delay(13).unwrap();

        assert!(matches!(plan.strategy, LoopStrategy::Bit8));
        assert_eq!(plan.iterations, 1);
        assert_eq!(plan.remaining_cycles, 0);
    }

    #[test]
    fn test_bit8_100_cycles() {
        let plan = plan_delay(100).unwrap();

        assert!(matches!(plan.strategy, LoopStrategy::Bit8));
        assert_eq!(plan.iterations, 30);
        assert_eq!(plan.remaining_cycles, 0);
    }

    #[test]
    fn test_bit8_500_cycles() {
        let plan = plan_delay(500).unwrap();

        assert!(matches!(plan.strategy, LoopStrategy::Bit8));
        assert_eq!(plan.iterations, 163);
        assert_eq!(plan.remaining_cycles, 1);
    }

    #[test]
    fn test_bit8_maximum() {
        let plan = plan_delay(778).unwrap();

        assert!(matches!(plan.strategy, LoopStrategy::Bit8));
        assert_eq!(plan.iterations, 256);
        assert_eq!(plan.remaining_cycles, 0);
    }

    #[test]
    fn test_bit16_1000_cycles() {
        let plan = plan_delay(1000).unwrap();

        assert!(matches!(plan.strategy, LoopStrategy::Bit16));
        assert_eq!(plan.iterations, 246);
        assert_eq!(plan.remaining_cycles, 0);
    }

    #[test]
    fn test_bit16_maximum() {
        let plan = plan_delay(262_161).unwrap();

        assert!(matches!(plan.strategy, LoopStrategy::Bit16));
        assert_eq!(plan.iterations, 65_536);
        assert_eq!(plan.remaining_cycles, 1);
    }

    #[test]
    fn test_nested_not_implemented() {
        assert!(plan_delay(300_000).is_err());
    }

    #[test]
    fn test_zero_cycles() {
        let plan = plan_delay(0).unwrap();

        assert!(matches!(plan.strategy, LoopStrategy::PaddingOnly));
        assert_eq!(plan.iterations, 0);
        assert_eq!(plan.remaining_cycles, 0);
    }

    #[test]
    fn test_total_cycle_accounting() {
        for target in [13, 100, 500, 778, 1000, 262_161] {
            let plan = plan_delay(target).unwrap();

            let loop_cycles = match plan.strategy {
                LoopStrategy::Bit8 => {
                    loop8_cycles(u16::try_from(plan.iterations).unwrap()).unwrap()
                }

                LoopStrategy::Bit16 => {
                    loop16_cycles(plan.iterations).unwrap()
                }

                LoopStrategy::PaddingOnly => 0,
                LoopStrategy::Nested => unreachable!(),
            };

            assert_eq!(
                loop_cycles + plan.overhead_cycles + plan.remaining_cycles,
                target,
                "Incorrect cycle accounting for target {target}"
            );
        }
    }
}