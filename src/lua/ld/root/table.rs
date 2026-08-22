use mlua::{Lua, Table};

use super::super::table::{Builder, build};
use super::constants::{RULES, TASK};
use super::{rules, task};

pub fn table(lua: &Lua) -> mlua::Result<Table> {
    let functions: [(&str, Builder); 2] = [(RULES, rules::function), (TASK, task::function)];

    build(lua, &functions)
}
