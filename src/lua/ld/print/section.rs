use mlua::{Function, Lua, Table, Value};

use super::constants::{NAMESPACE, SECTION};
use super::parse::{message, text};
use crate::output::{self, Message, Tone};

pub fn function(lua: &Lua) -> mlua::Result<Function> {
    lua.create_function(|lua, (value, options): (Value, Option<Table>)| {
        let call = format!("{NAMESPACE}.{SECTION}");
        let base = Message::new(text(&call, &value)?)
            .with_look(Tone::Strong.into())
            .with_blank(true);
        output::say(&message(lua, &call, base, options)?);

        Ok(())
    })
}
