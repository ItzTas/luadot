use mlua::{Function, Lua, Table, Value};

use super::constants::{ENTRY, NAMESPACE};
use super::parse::{message, text};
use crate::output::{self, LABEL_WIDTH, Message};

pub fn function(lua: &Lua) -> mlua::Result<Function> {
    lua.create_function(
        |lua, (label, value, options): (Value, Value, Option<Table>)| {
            let call = format!("{NAMESPACE}.{ENTRY}");
            let base = Message::new(text(&call, &label)?)
                .with_tail(text(&call, &value)?)
                .with_column(Some(LABEL_WIDTH));
            output::say(&message(lua, &call, base, options)?);

            Ok(())
        },
    )
}
