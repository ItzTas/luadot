mod commands;
mod run;
mod types;

use std::collections::HashMap;

pub use run::run;
pub use types::Command;

pub fn get_commands() -> HashMap<&'static str, Command> {
    let mut map: HashMap<&'static str, Command> = HashMap::new();
    map.insert("add", Command::Run(commands::add_cmd));
    map.insert("clone", Command::Run(commands::clone_cmd));
    map.insert("edit", Command::Run(commands::edit_cmd));
    map.insert("git", Command::Run(commands::git_cmd));
    map.insert("push", Command::Run(commands::push_cmd));
    map.insert("sync", Command::Run(commands::sync_cmd));
    return map;
}
