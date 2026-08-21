use std::env;

use mlua::{Lua, Table};

use super::constants::{ARGS, NAME};

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
    argv.set(NAME, name)?;
    argv.set(ARGS, lua.create_sequence_from(rest.iter().cloned())?)?;

    Ok(argv)
}
