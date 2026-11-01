use crate::planner::{DelayPlan, LoopStrategy};

pub fn generate_assembly(plan: &DelayPlan) -> String {
    let mut assembly = String::new();

    match plan.strategy {
        LoopStrategy::PaddingOnly => {

        }

        LoopStrategy::Bit8 => {

        }

        LoopStrategy::Bit16 => {

        }

        LoopStrategy::Nested => {

        }
    }

    for _ in 0..plan.remaining_cycles {
        assembly.push_str("nop\n");
    }

    assembly
}