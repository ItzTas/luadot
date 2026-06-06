mod cli;
mod files;
mod git;
mod lua;
mod state;
mod utils;

use std::process::ExitCode;

fn main() -> ExitCode {
    match cli::run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("luadot: {err:#}");
            ExitCode::FAILURE
        }
    }
}
