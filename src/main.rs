#![allow(warnings)]

mod cli;
mod usage;
mod generate_command;

use crate::cli::dispatch_command;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    let Some((command, command_args)) = args.split_first() else {
        eprintln!("Error: No command provided");
        std::process::exit(1);
    };
    match dispatch_command(command, command_args) {
        Ok(()) => {}
        Err(error) => {
            eprintln!("Error: {error:?}");
            std::process::exit(1);
        }
    }
}