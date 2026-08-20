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
    fn reports_a_text_that_is_not_a_string() {
        let err = run("print_({})").unwrap_err().to_string();

        assert!(err.contains("`ld.print` takes a string, got table"));
    }
}
