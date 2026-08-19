use mlua::{Function, Lua, Table, Value};

use super::constants::{NAMESPACE, WARN};
use super::parse::{message, text};
use crate::output::{self, Message, Stream, Tone, notice};

pub fn function(lua: &Lua) -> mlua::Result<Function> {
    lua.create_function(|lua, (value, options): (Value, Option<Table>)| {
        let call = format!("{NAMESPACE}.{WARN}");
        let base = Message::new(notice(text(&call, &value)?))
            .with_look(Tone::Warning.into())
            .with_stream(Stream::Stderr);
        output::say(&message(lua, &call, base, options)?);

        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::super::parse::message;
    use super::super::table::table;
    use crate::lua::runtime::runtime;
    use crate::output::{Message, Stream, Tone};

    fn run(source: &str) -> mlua::Result<()> {
        let lua = runtime().unwrap();
        lua.globals().set("print_", table(&lua).unwrap()).unwrap();

        lua.load(source).exec()
    }

    #[test]
    fn a_warning_goes_to_the_error_stream() {
        assert!(run(r#"print_.warn("careful")"#).is_ok());
    }

    #[test]
    fn the_options_win_over_the_tone_of_the_call() {
        let lua = runtime().unwrap();
        let options = lua.load(r#"return { tone = "good" }"#).eval().unwrap();
        let base = Message::new("careful")
            .with_look(Tone::Warning.into())
            .with_stream(Stream::Stderr);

        let message = message(&lua, "print.warn", base, Some(options)).unwrap();

        assert_eq!(message.look().style(), Tone::Good.style());
        assert_eq!(message.stream(), Stream::Stderr);
    }
}
