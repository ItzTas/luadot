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
    fn a_note_carries_the_name_of_the_program() {
        assert!(run(r#"print_.note("nothing is managed")"#).is_ok());
    }
}
