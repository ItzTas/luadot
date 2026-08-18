use mlua::{Function, Lua, Table};

use super::super::surface::{self, Surface};
use super::constants::{NAMESPACE, REPORT_KEYS, STATUS};
use super::parse::{known, report};
use crate::lua::Config;

pub fn function(lua: &Lua) -> mlua::Result<Function> {
    lua.create_function(|lua, options: Table| {
        let call = format!("{NAMESPACE}.{STATUS}");
        if surface::inert(lua, &call, Surface::Config) {
            return Ok(());
        }

        known(&call, &options, &REPORT_KEYS)?;
        let status = report(&call, &options)?;
        Config::building(lua)?.set_status(status);

        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use crate::lua::{Custom, StatusCounts, from_source};

    #[test]
    fn nothing_is_customized_until_the_call_runs() {
        let config = from_source("local unused = 1").unwrap();
        let status = config.status();

        assert!(status.entry().is_none());
        assert!(status.summary().is_none());
        assert!(status.render().is_none());
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
    fn false_silences_what_it_is_given_to() {
        let config =
            from_source("ld.on.status({ summary = false, entry = false, render = false })")
                .unwrap();

        assert!(matches!(config.status().summary(), Some(Custom::Silent)));
        assert!(matches!(config.status().entry(), Some(Custom::Silent)));
        assert!(matches!(config.status().render(), Some(Custom::Silent)));
    }

    #[test]
    fn a_later_call_only_replaces_the_keys_it_carries() {
        let config = from_source(
            r#"
            ld.on.status({ entry = false, summary = "first" })
            ld.on.status({ summary = "second" })
            "#,
        )
        .unwrap();

        assert!(matches!(config.status().entry(), Some(Custom::Silent)));
        assert!(matches!(config.status().summary(), Some(Custom::Text(text)) if text == "second"));
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
        assert!(err.contains("available: entry, render, summary"));
    }

    #[test]
    fn rejects_a_value_the_key_does_not_accept() {
        let err = format!(
            "{:#}",
            from_source(r#"ld.on.status({ entry = "a line" })"#).unwrap_err()
        );

        assert!(err.contains("`entry` takes a function or false"));
    }

    #[test]
    fn a_call_away_from_the_configuration_does_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let template = dir.path().join(".zshrc.luadot");
        std::fs::create_dir_all(&template).unwrap();

        assert!(
            crate::lua::from_template(
                &template,
                "ld.on.status({ summary = false })\nreturn \"ok\""
            )
            .is_ok()
        );
    }
}
