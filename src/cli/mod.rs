mod commands;
mod completions;
mod constants;
mod run;
mod types;

#[cfg(test)]
pub use commands::documented;
pub use run::run;
pub use types::Cli;
