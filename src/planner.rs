use crate::GenerationMode;
use crate::error::DelayError;
use crate::timing::{loop8_cycles, loop16_cycles};

const BIT_8_OVERHEAD_CYCLES: u64 = 10;
const BIT_16_OVERHEAD_CYCLES: u64 = 15;

#[derive(Debug)]
pub enum LoopStrategy {
    Bit8,
    Bit16,
    PaddingOnly,
    Nested {
        outer: Box<LoopStrategy>,
        inner: Box<LoopStrategy>,
    },
}

#[derive(Debug)]
pub struct DelayPlan {
    pub strategy: LoopStrategy,
    pub mode: GenerationMode,
    pub iterations: u32,
    pub remaining_cycles: u64,
    pub overhead_cycles: u64,
}

pub fn plan_delay(target_cycles: u64, mode: GenerationMode) -> Result<DelayPlan, DelayError> {
    let return_cycles = match mode {
        GenerationMode::Inline => 0,
        GenerationMode::Subroutine => 4,
    };
    let available_cycles = target_cycles
        .checked_sub(return_cycles)
        .ok_or(DelayError::UnachievableDelay)?;

    let strategy = if available_cycles < 13 {
        LoopStrategy::PaddingOnly
    } else if available_cycles <= 778 {
        LoopStrategy::Bit8
    } else if available_cycles <= 262_161 {
        LoopStrategy::Bit16
    } else {
        LoopStrategy::Nested {
            outer: Box::new(LoopStrategy::Bit16),
            inner: Box::new(LoopStrategy::Bit16),
        }
    };

    let overhead_cycles = match &strategy {
        LoopStrategy::Bit8 => BIT_8_OVERHEAD_CYCLES,
        LoopStrategy::Bit16 => BIT_16_OVERHEAD_CYCLES,
        LoopStrategy::PaddingOnly => 0,
        LoopStrategy::Nested { .. } => {
            return Err(DelayError::UnachievableDelay);
        }
    };

    let number_of_iterations = match &strategy {
        LoopStrategy::Bit8 => (available_cycles - BIT_8_OVERHEAD_CYCLES) / 3,
        LoopStrategy::Bit16 => (available_cycles - BIT_16_OVERHEAD_CYCLES - 1) / 4,
        LoopStrategy::PaddingOnly => 0,
        LoopStrategy::Nested { .. } => {
            return Err(DelayError::UnachievableDelay);
        }
    };

    let loop_cycles = match &strategy {
        LoopStrategy::Bit8 => {
            let iterations =
                u16::try_from(number_of_iterations).map_err(|_| DelayError::CycleOverflow)?;
            loop8_cycles(iterations)?
        }

        LoopStrategy::Bit16 => {
            let iterations =
                u32::try_from(number_of_iterations).map_err(|_| DelayError::CycleOverflow)?;
            loop16_cycles(iterations)?
        }

        LoopStrategy::PaddingOnly => 0,

        LoopStrategy::Nested { .. } => {
            return Err(DelayError::UnachievableDelay);
        }
    };

    let remaining_cycles = available_cycles
        .checked_sub(loop_cycles + overhead_cycles)
        .ok_or(DelayError::UnachievableDelay)?;
    let iterations = u32::try_from(number_of_iterations).map_err(|_| DelayError::CycleOverflow)?;

    Ok(DelayPlan {
        strategy,
        mode,
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
        let plan = plan_delay(5, GenerationMode::Inline).unwrap();

        assert!(matches!(plan.strategy, LoopStrategy::PaddingOnly));
        assert_eq!(plan.iterations, 0);
        assert_eq!(plan.remaining_cycles, 5);
        assert_eq!(plan.overhead_cycles, 0);
    }

    #[test]
    fn test_bit8_minimum() {
        let plan = plan_delay(13, GenerationMode::Inline).unwrap();

        assert!(matches!(plan.strategy, LoopStrategy::Bit8));
        assert_eq!(plan.iterations, 1);
        assert_eq!(plan.remaining_cycles, 0);
    }

    #[test]
    fn test_bit8_100_cycles() {
        let plan = plan_delay(100, GenerationMode::Inline).unwrap();

        assert!(matches!(plan.strategy, LoopStrategy::Bit8));
        assert_eq!(plan.iterations, 30);
        assert_eq!(plan.remaining_cycles, 0);
    }

    #[test]
    fn test_bit8_500_cycles() {
        let plan = plan_delay(500, GenerationMode::Inline).unwrap();

        assert!(matches!(plan.strategy, LoopStrategy::Bit8));
        assert_eq!(plan.iterations, 163);
        assert_eq!(plan.remaining_cycles, 1);
    }

    #[test]
    fn test_bit8_maximum() {
        let plan = plan_delay(778, GenerationMode::Inline).unwrap();

        assert!(matches!(plan.strategy, LoopStrategy::Bit8));
        assert_eq!(plan.iterations, 256);
        assert_eq!(plan.remaining_cycles, 0);
    }

    #[test]
    fn test_bit16_1000_cycles() {
        let plan = plan_delay(1000, GenerationMode::Inline).unwrap();

        assert!(matches!(plan.strategy, LoopStrategy::Bit16));
        assert_eq!(plan.iterations, 246);
        assert_eq!(plan.remaining_cycles, 0);
    }

    #[test]
    fn test_bit16_maximum() {
        let plan = plan_delay(262_161, GenerationMode::Inline).unwrap();

        assert!(matches!(plan.strategy, LoopStrategy::Bit16));
        assert_eq!(plan.iterations, 65_536);
        assert_eq!(plan.remaining_cycles, 1);
    }

    #[test]
    fn test_nested_not_implemented() {
        assert!(plan_delay(300_000, GenerationMode::Inline).is_err());
    }

    #[test]
    fn test_zero_cycles_inline() {
        let plan = plan_delay(0, GenerationMode::Inline).unwrap();

        assert!(matches!(plan.strategy, LoopStrategy::PaddingOnly));
        assert_eq!(plan.iterations, 0);
        assert_eq!(plan.remaining_cycles, 0);
    }

    #[test]
    fn test_subroutine_minimum() {
        let plan = plan_delay(4, GenerationMode::Subroutine).unwrap();

        assert!(matches!(plan.strategy, LoopStrategy::PaddingOnly));
        assert_eq!(plan.iterations, 0);
        assert_eq!(plan.remaining_cycles, 0);
    }

    #[test]
    fn test_subroutine_too_short() {
        assert!(plan_delay(0, GenerationMode::Subroutine).is_err());
        assert!(plan_delay(3, GenerationMode::Subroutine).is_err());
    }

    #[test]
    fn test_subroutine_five_cycles() {
        let plan = plan_delay(5, GenerationMode::Subroutine).unwrap();

        assert!(matches!(plan.strategy, LoopStrategy::PaddingOnly));
        assert_eq!(plan.iterations, 0);
        assert_eq!(plan.remaining_cycles, 1);
    }

    #[test]
    fn test_subroutine_13_cycles() {
        let plan = plan_delay(13, GenerationMode::Subroutine).unwrap();

        assert!(matches!(plan.strategy, LoopStrategy::PaddingOnly));
        assert_eq!(plan.iterations, 0);
        assert_eq!(plan.remaining_cycles, 9);
    }

    #[test]
    fn test_subroutine_bit8_minimum() {
        let plan = plan_delay(17, GenerationMode::Subroutine).unwrap();

        assert!(matches!(plan.strategy, LoopStrategy::Bit8));
        assert_eq!(plan.iterations, 1);
        assert_eq!(plan.remaining_cycles, 0);
    }

    #[test]
    fn test_subroutine_100_cycles() {
        let plan = plan_delay(100, GenerationMode::Subroutine).unwrap();

        assert!(matches!(plan.strategy, LoopStrategy::Bit8));
        assert_eq!(plan.iterations, 28);
        assert_eq!(plan.remaining_cycles, 2);
    }

    #[test]
    fn test_total_cycle_accounting() {
        let cases = [
            (0, GenerationMode::Inline),
            (5, GenerationMode::Inline),
            (13, GenerationMode::Inline),
            (100, GenerationMode::Inline),
            (500, GenerationMode::Inline),
            (778, GenerationMode::Inline),
            (1000, GenerationMode::Inline),
            (262_161, GenerationMode::Inline),
            (4, GenerationMode::Subroutine),
            (5, GenerationMode::Subroutine),
            (13, GenerationMode::Subroutine),
            (17, GenerationMode::Subroutine),
            (100, GenerationMode::Subroutine),
            (1000, GenerationMode::Subroutine),
        ];

        for (target, mode) in cases {
            let plan = plan_delay(target, mode).unwrap();

            let loop_cycles = match &plan.strategy {
                LoopStrategy::Bit8 => {
                    loop8_cycles(u16::try_from(plan.iterations).unwrap()).unwrap()
                }

                LoopStrategy::Bit16 => loop16_cycles(plan.iterations).unwrap(),

                LoopStrategy::PaddingOnly => 0,
                LoopStrategy::Nested { .. } => unreachable!(),
            };

            let return_cycles = match mode {
                GenerationMode::Inline => 0,
                GenerationMode::Subroutine => 4,
            };

            assert_eq!(
                loop_cycles + plan.overhead_cycles + plan.remaining_cycles + return_cycles,
                target,
                "Incorrect cycle accounting for target {target}, mode {mode:?}"
            );
        }
    }
}
