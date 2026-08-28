use mlua::{Function, Lua, Table};

use super::command::Command;
use super::constants::{NAMESPACE, STATUS_KEYS};
use super::parse::{around, hints, known, report};
use crate::lua::Config;

pub fn function(lua: &Lua, command: Command) -> mlua::Result<Function> {
    lua.create_function(move |lua, options: Table| {
        let call = format!("{NAMESPACE}.{}", command.path());
        known(&call, &options, &STATUS_KEYS)?;
        let status = report(&call, &options)?;
        let around = around(&call, &options)?;
        let hints = hints(&call, &options)?;
        Config::building(lua, |config| {
            config.set_status(status);
            config.set_around(command, around);
            config.set_command_hints(command, hints);
        })
    })
}

#[cfg(test)]
mod tests {
    use crate::lua::{Command, Custom, Moment, StatusCounts, from_source};

    #[test]
    fn report_and_moments_live_together() {
        let config = from_source(
            r#"ld.on.status({ summary = false, after = function() return "done" end })"#,
        )
        .unwrap();

        assert!(matches!(config.status().summary(), Some(Custom::Silent)));
        assert!(matches!(
            config.around(Command::Status).unwrap().all(Moment::After),
            [Custom::Call(_)]
        ));
    }

    #[test]
    fn the_function_survives_the_run() {
        let config = from_source(
            r#"ld.on.status({ summary = function(counts)
                 return counts.synced .. "/" .. counts.total .. " synced"
               end })"#,
        )
        .unwrap();

        let counts = StatusCounts::new(crate::files::Side::Repository, 14, "unused".to_string())
            .with_states(vec![(crate::files::FileStatus::Synced, 12)]);
        let shown = config
            .status()
            .summary()
            .unwrap()
            .shown("a hook", &counts)
            .unwrap();

        assert_eq!(shown, Some("12/14 synced".to_string()));
    }
}
