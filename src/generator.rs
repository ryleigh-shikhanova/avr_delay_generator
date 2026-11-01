use crate::{GenerationMode, error::DelayError, planner::*};
use std::fmt::Write;

const INDENT: &str = "    ";

fn generate_preamble(assembly: &mut String, strategy: &LoopStrategy) {
    match strategy {
        LoopStrategy::PaddingOnly => {}
        LoopStrategy::Bit8 => {
            writeln!(assembly, "{INDENT}push r18").unwrap();
            writeln!(assembly, "{INDENT}in r18, 0x3F").unwrap();
            writeln!(assembly, "{INDENT}push r18\n").unwrap();
        }
        LoopStrategy::Bit16 => {
            writeln!(assembly, "{INDENT}push r24").unwrap();
            writeln!(assembly, "{INDENT}push r25").unwrap();
            writeln!(assembly, "{INDENT}in r24, 0x3F").unwrap();
            writeln!(assembly, "{INDENT}push r24\n").unwrap();
        }
        LoopStrategy::Nested16 => {}
    }
}

fn generate_postamble(assembly: &mut String, strategy: &LoopStrategy) {
    match strategy {
        LoopStrategy::PaddingOnly => {}
        LoopStrategy::Bit8 => {
            writeln!(assembly, "{INDENT}pop r18").unwrap();
            writeln!(assembly, "{INDENT}out 0x3F, r18").unwrap();
            writeln!(assembly, "{INDENT}pop r18").unwrap();
        }
        LoopStrategy::Bit16 => {
            writeln!(assembly, "{INDENT}pop r24").unwrap();
            writeln!(assembly, "{INDENT}out 0x3F, r24").unwrap();
            writeln!(assembly, "{INDENT}pop r25").unwrap();
            writeln!(assembly, "{INDENT}pop r24").unwrap();
        }
        LoopStrategy::Nested16 => {}
    }
}

pub fn generate_assembly(plan: &DelayPlan) -> Result<String, DelayError> {
    let mut assembly = String::new();
    let preserve_registers = !matches!(&plan.strategy, LoopStrategy::PaddingOnly);

    if matches!(plan.mode, GenerationMode::Subroutine) {
        writeln!(assembly, "DELAY_LOOP:").unwrap();
    }

    if preserve_registers {
        generate_preamble(&mut assembly, &plan.strategy);
    }

    match &plan.strategy {
        LoopStrategy::PaddingOnly => {}

        LoopStrategy::Bit8 => {
            let initial_value = plan.iterations % 256;

            writeln!(assembly, "{INDENT}ldi r18, {initial_value}").unwrap();
            writeln!(assembly, "AGAIN:").unwrap();
            writeln!(assembly, "{INDENT}dec r18").unwrap();
            writeln!(assembly, "{INDENT}brne AGAIN\n").unwrap();
        }

        LoopStrategy::Bit16 => {
            let initial_value = plan.iterations % 65_536;

            writeln!(assembly, "{INDENT}ldi r24, lo8({initial_value})").unwrap();
            writeln!(assembly, "{INDENT}ldi r25, hi8({initial_value})").unwrap();
            writeln!(assembly, "AGAIN:").unwrap();
            writeln!(assembly, "{INDENT}sbiw r24, 1").unwrap();
            writeln!(assembly, "{INDENT}brne AGAIN\n").unwrap();
        }

        LoopStrategy::Nested16 => {
            return Err(DelayError::UnachievableDelay);
        }
    }

    for _ in 0..plan.remaining_cycles {
        assembly.push_str("nop\n");
    }

    if preserve_registers {
        generate_postamble(&mut assembly, &plan.strategy);
    }

    if matches!(plan.mode, GenerationMode::Subroutine) {
        writeln!(assembly, "{INDENT}ret").unwrap(); //ret consumes 4 cycles
    }

    Ok(assembly)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_padding_only_generation() {
        let plan = plan_delay(5, GenerationMode::Inline).unwrap();
        let assembly = generate_assembly(&plan).unwrap();

        assert_eq!(assembly, "nop\n".repeat(5));
    }

    #[test]
    fn test_bit16_generation() {
        let plan = plan_delay(1000, GenerationMode::Subroutine).unwrap();
        let assembly = generate_assembly(&plan).unwrap();

        assert!(assembly.contains("ldi r24, lo8("));
        assert!(assembly.contains("ldi r25, hi8("));
        assert!(assembly.contains("sbiw r24, 1"));
        assert!(assembly.contains("brne AGAIN"));
        assert!(assembly.contains("out 0x3F, r24"));
        assert!(assembly.contains("ret"));
    }
}
