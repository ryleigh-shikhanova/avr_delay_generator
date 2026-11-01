mod cli;
mod usage;

use cli::{CliCommand, dispatch_command};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
}
