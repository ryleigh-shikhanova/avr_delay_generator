use crate::cli::CommandError;

use avr_delay_generator::{
    DelayConfig, GenerationMode, RoundingMode, generator::generate_assembly, planner::{DelayPlan, plan_delay}, timing::delay_to_cycles,
};

const MIN_ARGS: usize = 2;
const MAX_ARGS: usize = 2;

const RELATIVE_CONVERSION_TO_NS: &[(&str, u64)] = &[
    ("ms", 1_000_000),
    ("s", 1_000_000_000),
    ("us", 1_000),
    ("ns", 1),
];

pub fn convert_to_ns(arg: &str) -> Result<u64, CommandError>{
    let (left, right) = arg
        .find(|c: char| c.is_ascii_alphabetic())
        .map(|idx| arg.split_at(idx))
        .ok_or(CommandError::InvalidArgument)?;

    let numeric_arg: u64 = left
        .parse::<u64>()
        .map_err(|_| CommandError::InvalidArgument)?;

    let multiplier = RELATIVE_CONVERSION_TO_NS
        .iter()
        .find(|(unit, _)| *unit == right.trim())
        .map(|(_, multiplier)| *multiplier)
        .ok_or(CommandError::InvalidArgument)?;

    numeric_arg
        .checked_mul(multiplier)
        .ok_or(CommandError::InvalidArgument)
}

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

    
    let delay_ns = convert_to_ns(&args[1])?;

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