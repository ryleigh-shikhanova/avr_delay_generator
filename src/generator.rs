use crate::{GenerationMode, error::DelayError, planner::*};
use std::fmt::Write;

const INDENT: &str = "    ";

fn generate_preamble(assembly: &mut String) {
    writeln!(assembly, "{INDENT}push r18").unwrap();
    writeln!(assembly, "{INDENT}in r18, 0x3F").unwrap();
    writeln!(assembly, "{INDENT}push r18\n").unwrap();
}

fn generate_postamble(assembly: &mut String) {
    writeln!(assembly, "{INDENT}pop r18").unwrap();
    writeln!(assembly, "{INDENT}out 0x3F, r18").unwrap();
    writeln!(assembly, "{INDENT}pop r18").unwrap();
}

pub fn generate_assembly(plan: &DelayPlan) -> Result<String, DelayError> {
    let mut assembly = String::new();
    let preserve_registers = !matches!(&plan.strategy, LoopStrategy::PaddingOnly);

    if preserve_registers {
        generate_preamble(&mut assembly);
    }

    match &plan.strategy {
        LoopStrategy::PaddingOnly => {}

        LoopStrategy::Bit8 => {
            let initial_value = plan.iterations % 256;

            writeln!(assembly, "{INDENT}ldi r18, {initial_value}").unwrap();
            writeln!(assembly, "DELAY_LOOP:").unwrap();
            writeln!(assembly, "{INDENT}dec r18").unwrap();
            writeln!(assembly, "{INDENT}brne DELAY_LOOP\n").unwrap();
        }

        LoopStrategy::Bit16 => {
            return Err(DelayError::UnachievableDelay);
        }

        LoopStrategy::Nested { .. } => {
            return Err(DelayError::UnachievableDelay);
        }
    }

    for _ in 0..plan.remaining_cycles {
        assembly.push_str("nop\n");
    }

    if preserve_registers {
        generate_postamble(&mut assembly);
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
}
