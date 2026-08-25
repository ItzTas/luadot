use mlua::{Lua, Table};

use super::command::Command;
use super::constants::{Customizer, FUNCTIONS, TMPL, TMPL_FUNCTIONS};

pub fn table(lua: &Lua) -> mlua::Result<Table> {
    let table = build(lua, &FUNCTIONS)?;
    table.set(TMPL, build(lua, &TMPL_FUNCTIONS)?)?;

    Ok(table)
}

fn build(lua: &Lua, functions: &[(&str, Command, Customizer)]) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    for (name, command, function) in functions {
        table.set(*name, function(lua, *command)?)?;
    }

    Ok(table)
}
