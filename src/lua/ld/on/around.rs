use mlua::{Function, Lua, Table};

use super::command::Command;
use super::constants::{AROUND_KEYS, NAMESPACE};
use super::parse::{around, known};
use crate::lua::Config;

pub fn function(lua: &Lua, command: Command) -> mlua::Result<Function> {
    lua.create_function(move |lua, options: Table| {
        let call = format!("{NAMESPACE}.{}", command.path());
        known(&call, &options, &AROUND_KEYS)?;
        let around = around(&call, &options)?;
        Config::building(lua, |config| config.set_around(command, around))
    })
}

#[cfg(test)]
mod tests {
    use crate::lua::{Command, Custom, Moment, from_source};

    #[test]
    fn a_function_is_kept_for_the_command_it_was_set_on() {
        let config = from_source(
            r#"ld.on.apply({ before = function() return "applying on " .. ld.sys.host.name end })"#,
        )
        .unwrap();

        let shown = config
            .around(Command::Apply)
            .unwrap()
            .get(Moment::Before)
            .unwrap()
            .shown("a function", ())
            .unwrap()
            .unwrap();

        assert!(shown.starts_with("applying on "));
        assert!(config.around(Command::Add).is_none());
    }

    #[test]
    fn the_tmpl_actions_are_customized_apart() {
        let config = from_source(
            r#"
            ld.on.tmpl.alt({ after = false })
            ld.on.tmpl.new({ before = function() end })
            "#,
        )
        .unwrap();

        let alt = config.around(Command::TmplAlt).unwrap();
        let new = config.around(Command::TmplNew).unwrap();

        assert!(matches!(alt.get(Moment::After), Some(Custom::Silent)));
        assert!(alt.get(Moment::Before).is_none());
        assert!(matches!(new.get(Moment::Before), Some(Custom::Call(_))));
        assert!(new.get(Moment::After).is_none());
    }
}
