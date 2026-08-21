use mlua::{Function, Lua, Table};

use super::command::Command;
use super::constants::{NAMESPACE, STATUS_KEYS};
use super::parse::{around, known, report};
use crate::lua::Config;

pub fn function(lua: &Lua, command: Command) -> mlua::Result<Function> {
    lua.create_function(move |lua, options: Table| {
        let call = format!("{NAMESPACE}.{}", command.path());
        known(&call, &options, &STATUS_KEYS)?;
        let status = report(&call, &options)?;
        let around = around(&call, &options)?;
        Config::building(lua, |config| {
            config.set_status(status);
            config.set_around(command, around);
        })
    })
}

#[cfg(test)]
mod tests {
    use crate::lua::{Command, Custom, Moment, StatusCounts, from_source};

    #[test]
    fn the_report_and_the_moments_are_kept_side_by_side() {
        let config = from_source(
            r#"ld.on.status({ summary = false, after = function() return "done" end })"#,
        )
        .unwrap();

        assert!(matches!(config.status().summary(), Some(Custom::Silent)));
        assert!(matches!(
            config.around(Command::Status).unwrap().get(Moment::After),
            Some(Custom::Call(_))
        ));
    }

    #[test]
    fn a_function_is_kept_callable_after_the_configuration_ran() {
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

    #[test]
    fn the_two_commands_are_customized_apart() {
        let config = from_source(
            r#"
            ld.on.diff({ summary = "diffed" })
            ld.on.status({ summary = "reported" })
            "#,
        )
        .unwrap();

        assert!(matches!(config.diff().summary(), Some(Custom::Text(text)) if text == "diffed"));
        assert!(
            matches!(config.status().summary(), Some(Custom::Text(text)) if text == "reported")
        );
    }

    #[test]
    fn rejects_the_keys_that_belong_to_another_command() {
        let err = format!(
            "{:#}",
            from_source(r#"ld.on.status({ tool = "difft" })"#).unwrap_err()
        );

        assert!(err.contains("`ld.on.status`: unknown key `tool`"));
        assert!(err.contains("available: after, before, entry, render, summary"));
    }

    #[test]
    fn rejects_a_value_the_key_does_not_accept() {
        let err = format!(
            "{:#}",
            from_source(r#"ld.on.status({ entry = "a line" })"#).unwrap_err()
        );

        assert!(err.contains("`entry` takes a function or false"));
    }
}
