use mlua::{Function, Lua, Value};

use super::constants::{PATTERN, TEST, TEXT};
use super::parse::{compile, text};

pub fn function(lua: &Lua) -> mlua::Result<Function> {
    lua.create_function(|_, (subject, pattern): (Value, Value)| {
        let subject = text(&subject, TEST, TEXT)?;
        let pattern = text(&pattern, TEST, PATTERN)?;

        Ok(compile(&pattern, TEST)?.is_match(&subject))
    })
}
