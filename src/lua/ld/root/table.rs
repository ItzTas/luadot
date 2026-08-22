use mlua::{Lua, Table};

use super::super::surface::Surface;
use super::super::table::{Builder, build};
use super::constants::{RULES, SURFACE, TASK};
use super::{rules, task};

pub fn table(lua: &Lua, surface: Surface) -> mlua::Result<Table> {
    let functions: [(&str, Builder); 2] = [(RULES, rules::function), (TASK, task::function)];

    let table = build(lua, &functions)?;
    table.set(SURFACE, surface.name())?;

    Ok(table)
}
