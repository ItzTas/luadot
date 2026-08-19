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

#[cfg(test)]
mod tests {
    use super::super::table::table;
    use crate::lua::runtime::runtime;

    fn run(source: &str) -> mlua::Result<()> {
        let lua = runtime().unwrap();
        lua.globals().set("print_", table(&lua).unwrap()).unwrap();

        lua.load(source).exec()
    }

    #[test]
    fn an_error_carries_the_name_of_the_program() {
        assert!(run(r#"print_.error("broken")"#).is_ok());
    }
}
