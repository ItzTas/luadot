pub mod constants;
mod load;
mod types;

pub use load::{config_path, load_config};
#[cfg(test)]
pub use load::{from_classes, from_source};
pub use types::{Class, Config, Rule};
