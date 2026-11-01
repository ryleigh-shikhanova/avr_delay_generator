use crate::cli::CommandError;

use avr_delay_generator::{
    DelayConfig,
    GenerationMode,
    RoundingMode,
    timing::delay_to_cycles,
    planner::plan_delay,
    generator::generate_assembly,
};

pub fn handle_generate_args(args: &[String]) -> Result<(), CommandError> {
    todo!()
}