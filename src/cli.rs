use crate::usage::handle_usage_args;
use avr_delay_generator::DelayConfig;

pub enum CliCommand {
    Help,
    _Generate(DelayConfig),
}

#[derive(Debug)]
pub enum CommandError {
    UnknownCommand,
    _NotEnoughArgs,
    _TooManyArgs,
}

pub fn dispatch_command(command: &str, args: &[String]) -> Result<(), CommandError> {
    match command {
        "h" | "help" => handle_usage_args(args),
        "g" | "generate" => todo!(),
        _ => Err(CommandError::UnknownCommand),
    }
}
