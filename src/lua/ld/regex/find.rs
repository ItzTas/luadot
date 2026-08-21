use mlua::{Function, Lua, Value, Variadic};

use super::constants::{FIND, PATTERN, TEXT};
use super::parse::{compile, text};

pub fn function(lua: &Lua) -> mlua::Result<Function> {
    lua.create_function(|_, (subject, pattern): (Value, Value)| {
        let subject = text(&subject, FIND, TEXT)?;
        let pattern = text(&pattern, FIND, PATTERN)?;

        let regex = compile(&pattern, FIND)?;
        let Some(found) = regex.find(&subject) else {
            return Ok(Variadic::new());
        };

        Ok(Variadic::from_iter([
            Value::Integer(found.start() as i64 + 1),
            Value::Integer(found.end() as i64),
        ]))
    })
}
