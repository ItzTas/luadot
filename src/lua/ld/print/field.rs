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
    fn a_field_takes_a_name_and_its_value() {
        assert!(run(r#"print_.field("repository", "/data/repo")"#).is_ok());
    }

    #[test]
    fn reports_a_name_that_is_not_a_string() {
        let err = run(r#"print_.field({}, "/data/repo")"#)
            .unwrap_err()
            .to_string();

        assert!(err.contains("`ld.print.field` takes a string, got table"));
    }
}
