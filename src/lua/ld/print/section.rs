use mlua::{Function, Lua, Table, Value};

use super::constants::{NAMESPACE, SECTION};
use super::parse::{message, text};
use crate::output::{self, Message, Tone};

pub fn function(lua: &Lua) -> mlua::Result<Function> {
    lua.create_function(|lua, (value, options): (Value, Option<Table>)| {
        let call = format!("{NAMESPACE}.{SECTION}");
        let base = Message::new(text(&call, &value)?)
            .with_look(Tone::Strong.into())
            .with_blank(true);
        output::say(&message(lua, &call, base, options)?);

        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::super::parse::message;
    use crate::lua::runtime::runtime;
    use crate::output::{Message, Tone};

    #[test]
    fn a_section_opens_with_a_blank_line_and_stands_out() {
        let lua = runtime().unwrap();
        let base = Message::new("Repository")
            .with_look(Tone::Strong.into())
            .with_blank(true);

        let message = message(&lua, "print.section", base, None).unwrap();

        assert!(message.blank());
        assert_eq!(message.look().style(), Tone::Strong.style());
    }
}
