use mlua::{Function, Lua, Value};

use super::constants::{ESCAPE, TEXT};
use super::parse::text;

pub fn function(lua: &Lua) -> mlua::Result<Function> {
    lua.create_function(|_, subject: Value| Ok(regex::escape(&text(&subject, ESCAPE, TEXT)?)))
}
