use std::env;

use mlua::{Lua, Table};

pub fn table(lua: &Lua) -> mlua::Result<Table> {
    from(lua, &invocation())
}

fn invocation() -> Vec<String> {
    strip_flags(env::args().skip(1))
}

fn strip_flags(args: impl Iterator<Item = String>) -> Vec<String> {
    args.skip_while(|arg| arg.starts_with('-')).collect()
}

fn from(lua: &Lua, args: &[String]) -> mlua::Result<Table> {
    let name = args.first().map(String::as_str).unwrap_or_default();
    let rest = args.get(1..).unwrap_or_default();

    let argv = lua.create_table()?;
    argv.set("name", name)?;
    argv.set("args", lua.create_sequence_from(rest.iter().cloned())?)?;

    Ok(argv)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lua::runtime::runtime;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| value.to_string()).collect()
    }

    #[test]
    fn exposes_the_command_and_what_follows_it() {
        let lua = runtime().unwrap();

        let argv = from(&lua, &args(&["apply", ".config/nvim", "--dry"])).unwrap();

        assert_eq!(argv.get::<String>("name").unwrap(), "apply");
        assert_eq!(
            argv.get::<Table>("args")
                .unwrap()
                .sequence_values::<String>()
                .collect::<mlua::Result<Vec<String>>>()
                .unwrap(),
            args(&[".config/nvim", "--dry"])
        );
    }

    #[test]
    fn a_command_without_arguments_yields_an_empty_list() {
        let lua = runtime().unwrap();

        let argv = from(&lua, &args(&["status"])).unwrap();

        assert_eq!(argv.get::<String>("name").unwrap(), "status");
        assert_eq!(argv.get::<Table>("args").unwrap().len().unwrap(), 0);
    }

    #[test]
    fn leading_flags_are_not_the_command() {
        assert_eq!(
            strip_flags(args(&["-v", "apply", "--dry-run"]).into_iter()),
            args(&["apply", "--dry-run"])
        );
        assert_eq!(strip_flags(args(&["apply"]).into_iter()), args(&["apply"]));
    }

    #[test]
    fn no_invocation_yields_an_empty_name() {
        let lua = runtime().unwrap();

        let argv = from(&lua, &[]).unwrap();

        assert_eq!(argv.get::<String>("name").unwrap(), "");
        assert_eq!(argv.get::<Table>("args").unwrap().len().unwrap(), 0);
    }

    #[test]
    fn the_process_invocation_builds_a_table() {
        let lua = runtime().unwrap();

        let argv = table(&lua).unwrap();

        assert!(argv.get::<String>("name").is_ok());
        assert!(argv.get::<Table>("args").is_ok());
    }
}
