mod commands;
mod run;
mod types;

use std::collections::HashMap;

pub use run::run;
pub use types::Command;

pub fn get_commands() -> HashMap<&'static str, Command> {
    let mut map: HashMap<&'static str, Command> = HashMap::new();
    map.insert("clone", Command::Run(commands::clone_cmd));
    map.insert("git", Command::Run(commands::git_cmd));
    return map;
}
