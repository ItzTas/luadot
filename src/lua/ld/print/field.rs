use mlua::{Function, Lua, Table, Value};

use super::constants::{FIELD, NAMESPACE};
use super::parse::{message, text};
use crate::output::{self, FIELD_WIDTH, Message, Tone};

pub fn function(lua: &Lua) -> mlua::Result<Function> {
    lua.create_function(
        |lua, (name, value, options): (Value, Value, Option<Table>)| {
            let call = format!("{NAMESPACE}.{FIELD}");
            let base = Message::new(text(&call, &name)?)
                .with_tail(text(&call, &value)?)
                .with_look(Tone::Strong.into())
                .with_column(Some(FIELD_WIDTH));
            output::say(&message(lua, &call, base, options)?);

            Ok(())
        },
    )
}
