use mlua::{Lua, Table};

use super::super::table::{Builder, build};
use super::constants::RULES;
use super::rules;

pub fn table(lua: &Lua) -> mlua::Result<Table> {
    let functions: [(&str, Builder); 1] = [(RULES, rules::function)];

    build(lua, &functions)
}
