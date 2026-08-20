use mlua::{Function, Lua, Table, Value};

use super::constants::{ERROR, NAMESPACE};
use super::parse::{message, text};
use crate::output::{self, Message, Stream, Tone, notice};

pub fn function(lua: &Lua) -> mlua::Result<Function> {
    lua.create_function(|lua, (value, options): (Value, Option<Table>)| {
        let call = format!("{NAMESPACE}.{ERROR}");
        let base = Message::new(notice(text(&call, &value)?))
            .with_look(Tone::Bad.into())
            .with_stream(Stream::Stderr);
        output::say(&message(lua, &call, base, options)?);

        Ok(())
    })
}
