use mlua::{Function, Lua};

use super::command::Command;
use super::{around, diff, status};
use crate::lua::config::constants::{AFTER, BEFORE};

pub const NAMESPACE: &str = "on";

pub const ADD: &str = "add";

pub const APPLY: &str = "apply";

pub const BOOTSTRAP: &str = "bootstrap";

pub const CD: &str = "cd";

pub const CLASS: &str = "class";

pub const CLONE: &str = "clone";

pub const CONFIG: &str = "config";

pub const DIFF: &str = "diff";

pub const EDIT: &str = "edit";

pub const EXEC: &str = "exec";

pub const GIT: &str = "git";

pub const INIT: &str = "init";

pub const MV: &str = "mv";

pub const PUSH: &str = "push";

pub const REKEY: &str = "rekey";

pub const RELINK: &str = "relink";

pub const RESTORE: &str = "restore";

pub const RM: &str = "rm";

pub const SETUP: &str = "setup";

pub const STATUS: &str = "status";

pub const SYNC: &str = "sync";

pub const TAKE: &str = "take";

pub const TMPL: &str = "tmpl";

pub const ALT: &str = "alt";

pub const NEW: &str = "new";

pub const TMPL_ALT: &str = "tmpl alt";

pub const TMPL_NEW: &str = "tmpl new";

pub type Customizer = fn(&Lua, Command) -> mlua::Result<Function>;

pub const FUNCTIONS: [(&str, Command, Customizer); 22] = [
    (ADD, Command::Add, around::function),
    (APPLY, Command::Apply, around::function),
    (BOOTSTRAP, Command::Bootstrap, around::function),
    (CD, Command::Cd, around::function),
    (CLASS, Command::Class, around::function),
    (CLONE, Command::Clone, around::function),
    (CONFIG, Command::Config, around::function),
    (DIFF, Command::Diff, diff::function),
    (EDIT, Command::Edit, around::function),
    (EXEC, Command::Exec, around::function),
    (GIT, Command::Git, around::function),
    (INIT, Command::Init, around::function),
    (MV, Command::Mv, around::function),
    (PUSH, Command::Push, around::function),
    (REKEY, Command::Rekey, around::function),
    (RELINK, Command::Relink, around::function),
    (RESTORE, Command::Restore, around::function),
    (RM, Command::Rm, around::function),
    (SETUP, Command::Setup, around::function),
    (STATUS, Command::Status, status::function),
    (SYNC, Command::Sync, around::function),
    (TAKE, Command::Take, around::function),
];

pub const TMPL_FUNCTIONS: [(&str, Command, Customizer); 2] = [
    (ALT, Command::TmplAlt, around::function),
    (NEW, Command::TmplNew, around::function),
];

pub const ARGS: &str = "args";

pub const ENTRY: &str = "entry";

pub const HINTS: &str = "hints";

pub const RENDER: &str = "render";

pub const SUMMARY: &str = "summary";

pub const TOOL: &str = "tool";

pub const AROUND_KEYS: [&str; 3] = [AFTER, BEFORE, HINTS];

pub const DIFF_KEYS: [&str; 8] = [AFTER, ARGS, BEFORE, ENTRY, HINTS, RENDER, SUMMARY, TOOL];

pub const STATUS_KEYS: [&str; 6] = [AFTER, BEFORE, ENTRY, HINTS, RENDER, SUMMARY];

#[cfg(test)]
mod tests {
    use super::*;

    fn registered() -> Vec<(String, Command)> {
        FUNCTIONS
            .iter()
            .map(|(name, command, _)| ((*name).to_string(), *command))
            .chain(
                TMPL_FUNCTIONS
                    .iter()
                    .map(|(name, command, _)| (format!("{TMPL}.{name}"), *command)),
            )
            .collect()
    }

    #[test]
    fn every_command_is_registered() {
        let mut commands: Vec<Command> = registered().into_iter().map(|(_, it)| it).collect();
        commands.sort();

        assert_eq!(commands, Command::ALL);
    }

    #[test]
    fn every_registered_name_is_its_command_path() {
        let mismatched: Vec<String> = registered()
            .into_iter()
            .filter(|(name, command)| *name != command.path())
            .map(|(name, command)| format!("{name} is registered for {command:?}"))
            .collect();

        assert_eq!(mismatched, Vec::<String>::new());
    }
}
