mod bootstrap;
mod config;
mod constants;
mod embed;
mod exec;
mod ld;
mod runtime;
mod script;
mod setup;
mod template;

pub use bootstrap::{bootstrap_path, run_bootstrap};
pub use config::{
    Call, Class, Config, Custom, Diff, DiffCounts, DiffFile, DiffState, Matcher, Report, Rule,
    StatusCounts, StatusFile, Tool, config_path, load_config,
};
#[cfg(test)]
pub use config::{from_classes, from_source};
pub use exec::run_exec;
pub use setup::{list_setups, run_setups};
pub use template::constants::TEMPLATE_FILE;
#[cfg(test)]
pub use template::from_source as from_template;
pub use template::{Content, Handle, Output, Template, load_template, load_template_file};
