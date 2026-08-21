use mlua::{Function, Lua, Value, Variadic};

use super::captures::{owned, values};
use super::constants::{MATCH, PATTERN, TEXT};
use super::parse::{compile, text};

pub fn function(lua: &Lua) -> mlua::Result<Function> {
    lua.create_function(|lua, (subject, pattern): (Value, Value)| {
        let subject = text(&subject, MATCH, TEXT)?;
        let pattern = text(&pattern, MATCH, PATTERN)?;

        let regex = compile(&pattern, MATCH)?;
        let Some(captures) = regex.captures(&subject) else {
            return Ok(Variadic::new());
        };

        values(lua, &owned(&captures))
    })
}
