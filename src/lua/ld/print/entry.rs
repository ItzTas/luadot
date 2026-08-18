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

#[cfg(test)]
mod tests {
    use super::super::parse::message;
    use super::super::table::table;
    use crate::lua::runtime::runtime;
    use crate::output::{LABEL_WIDTH, Message};

    fn run(source: &str) -> mlua::Result<()> {
        let lua = runtime().unwrap();
        lua.globals().set("print_", table(&lua).unwrap()).unwrap();

        lua.load(source).exec()
    }

    #[test]
    fn an_entry_takes_a_label_and_the_text_beside_it() {
        assert!(run(r#"print_.entry("create", "~/.bashrc", { tone = "good" })"#).is_ok());
    }

    #[test]
    fn the_label_fills_the_column_and_the_text_follows_it() {
        let lua = runtime().unwrap();
        let base = Message::new("create")
            .with_tail("~/.bashrc")
            .with_column(Some(LABEL_WIDTH));

        let message = message(&lua, "print.entry", base, None).unwrap();

        assert_eq!(message.head(), "create     ");
        assert_eq!(message.tail(), "~/.bashrc");
    }

    #[test]
    fn a_width_of_its_own_wins_over_the_column() {
        let lua = runtime().unwrap();
        let options = lua.load("return { width = 4 }").eval().unwrap();
        let base = Message::new("create").with_column(Some(LABEL_WIDTH));

        let message = message(&lua, "print.entry", base, Some(options)).unwrap();

        assert_eq!(message.head(), "create  ");
    }
}
