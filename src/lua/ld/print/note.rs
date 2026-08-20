use mlua::{Function, Lua, Table, Value};

use super::constants::{NAMESPACE, NOTE};
use super::parse::{message, text};
use crate::output::{self, Message, notice};

pub fn function(lua: &Lua) -> mlua::Result<Function> {
    lua.create_function(|lua, (value, options): (Value, Option<Table>)| {
        let call = format!("{NAMESPACE}.{NOTE}");
        let base = Message::new(notice(text(&call, &value)?));
        output::say(&message(lua, &call, base, options)?);

        Ok(())
    })
}
