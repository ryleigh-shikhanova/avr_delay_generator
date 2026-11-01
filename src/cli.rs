use crate::generate_command::handle_generate_args;
use crate::usage::handle_usage_args;
use avr_delay_generator::DelayConfig;

pub enum _CliCommand {
    Help,
    _Generate(DelayConfig),
}

#[derive(Debug)]
pub enum CommandError {
    UnknownCommand,
    NotEnoughArgs,
    TooManyArgs,
    InvalidArgument,
    GenerationFailed,
}

pub fn dispatch_command(command: &str, args: &[String]) -> Result<(), CommandError> {
    match command {
        "h" | "help" => handle_usage_args(args),
        "g" | "generate" => handle_generate_args(args),
        _ => Err(CommandError::UnknownCommand),
    }
}
