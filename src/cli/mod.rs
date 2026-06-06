mod commands;
mod types;

use std::collections::HashMap;

pub use types::Command;

pub fn get_commands() -> HashMap<&'static str, Command> {
    let mut map: HashMap<&'static str, Command> = HashMap::new();
    map.insert("clone", Command::Run(commands::clone));
    return map;
}
