use mlua::{Function, Lua, Value};

use super::constants::{PATTERN, SPLIT, TEXT};
use super::parse::{compile, limit, text};

pub fn function(lua: &Lua) -> mlua::Result<Function> {
    lua.create_function(|lua, (subject, pattern, pieces): (Value, Value, Value)| {
        let subject = text(&subject, SPLIT, TEXT)?;
        let pattern = text(&pattern, SPLIT, PATTERN)?;
        let pieces = limit(&pieces, SPLIT)?;

        let regex = compile(&pattern, SPLIT)?;
        let parts: Vec<&str> = match pieces {
            0 => regex.split(&subject).collect(),
            pieces => regex.splitn(&subject, pieces).collect(),
        };

        lua.create_sequence_from(parts)
    })
}
