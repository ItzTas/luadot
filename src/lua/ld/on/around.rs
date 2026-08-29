use mlua::{Function, Lua, Table};

use super::command::Command;
use super::constants::{AROUND_KEYS, NAMESPACE};
use super::parse::{around, hints, known};
use crate::lua::Config;

pub fn function(lua: &Lua, command: Command) -> mlua::Result<Function> {
    lua.create_function(move |lua, options: Table| {
        let call = format!("{NAMESPACE}.{}", command.path());
        known(&call, &options, &AROUND_KEYS)?;
        let around = around(&call, &options)?;
        let hints = hints(&call, &options)?;
        Config::building(lua, |config| {
            config.set_around(command, around);
            config.set_command_hints(command, hints);
        })
    })
}

#[cfg(test)]
mod tests {
    use crate::lua::{Command, Custom, Moment, from_source};

    #[test]
    fn a_function_is_kept_per_command() {
        let config = from_source(
            r#"ld.on.apply({ before = function() return "applying on " .. ld.path.home end })"#,
        )
        .unwrap();

        let shown = config.around(Command::Apply).unwrap().all(Moment::Before)[0]
            .shown("a function", ())
            .unwrap()
            .unwrap();

        assert!(shown.starts_with("applying on "));
        assert!(config.around(Command::Add).is_none());
    }

    #[test]
    fn functions_are_kept_in_order() {
        let config = from_source(
            r#"
            ld.on.apply({ before = function() return "first" end })
            ld.on.apply({ before = function() return "second" end, after = function() end })
            "#,
        )
        .unwrap();

        let chain = config.around(Command::Apply).unwrap();
        let shown: Vec<String> = chain
            .all(Moment::Before)
            .iter()
            .map(|custom| custom.shown("a function", ()).unwrap().unwrap())
            .collect();

        assert_eq!(shown, ["first", "second"]);
        assert_eq!(chain.all(Moment::After).len(), 1);
    }

    #[test]
    fn a_later_call_replaces_the_hints() {
        let config = from_source(
            r#"
            ld.on.apply({ hints = function() end })
            ld.on.apply({ hints = false })
            "#,
        )
        .unwrap();

        assert!(matches!(
            config.command_hints(Command::Apply),
            Some(Custom::Silent)
        ));
        assert!(config.command_hints(Command::Add).is_none());
    }

    #[test]
    fn tmpl_actions_are_kept_apart() {
        let config = from_source(
            r#"
            ld.on.tmpl.alt({ after = function() end })
            ld.on.tmpl.new({ before = function() end })
            "#,
        )
        .unwrap();

        let alt = config.around(Command::TmplAlt).unwrap();
        let new = config.around(Command::TmplNew).unwrap();

        assert!(matches!(alt.all(Moment::After), [Custom::Call(_)]));
        assert!(alt.all(Moment::Before).is_empty());
        assert!(matches!(new.all(Moment::Before), [Custom::Call(_)]));
        assert!(new.all(Moment::After).is_empty());
    }
}
