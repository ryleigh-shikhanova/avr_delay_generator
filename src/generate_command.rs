use crate::cli::CommandError;

use avr_delay_generator::{
    DelayConfig, GenerationMode, RoundingMode, generator::generate_assembly, planner::{DelayPlan, plan_delay}, timing::delay_to_cycles,
};

const MIN_ARGS: usize = 2;
const MAX_ARGS: usize = 2;

pub fn handle_generate_args(args: &[String]) -> Result<(), CommandError> {
    if args.len() < MIN_ARGS {
        return Err(CommandError::NotEnoughArgs);
    }

    if args.len() > MAX_ARGS {
        return Err(CommandError::TooManyArgs);
    }
    
    let clock_hz = args[0]
        .parse::<u64>()
        .map_err(|_| CommandError::InvalidArgument)?;

    let delay_ns = args[1]
        .parse::<u64>()
        .map_err(|_| CommandError::InvalidArgument)?;

    let config = DelayConfig {
        clock_hz,
        delay_ns,
        rounding_mode: RoundingMode::Nearest,
    };

    let target_cycles = delay_to_cycles(&config)
        .map_err(|_| CommandError::GenerationFailed)?;

    let mode = GenerationMode::Subroutine;

    let plan = plan_delay(target_cycles, mode)
        .map_err(|_| CommandError::GenerationFailed)?;

    let assembly = generate_assembly(&plan)
        .map_err(|_| CommandError::GenerationFailed)?;

    println!("{assembly}");

    Ok(())
}