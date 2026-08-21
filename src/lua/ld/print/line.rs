use mlua::{Function, Lua, Table, Value};

use super::constants::NAMESPACE;
use super::parse::{message, text};
use crate::output::{self, Message};

pub fn function(lua: &Lua) -> mlua::Result<Function> {
    lua.create_function(|lua, (_, value, options): (Table, Value, Option<Table>)| {
        let base = Message::new(text(NAMESPACE, &value)?);
        output::say(&message(lua, NAMESPACE, base, options)?);

        Ok(())
    })
}
