use mlua::{Function, Lua, Value};

use super::constants::{PATTERN, SPLIT, TEXT};
use super::parse::{compile, limit, text};

pub fn function(lua: &Lua) -> mlua::Result<Function> {
    lua.create_function(|lua, (subject, pattern, pieces): (Value, Value, Value)| {
        let subject = text(&subject, SPLIT, TEXT)?;
        let pattern = text(&pattern, SPLIT, PATTERN)?;
        let pieces = limit(&pieces, SPLIT)?;

        let regex = compile(&pattern, SPLIT)?;
        let parts: Vec<&str> = match pieces {
            0 => regex.split(&subject).collect(),
            pieces => regex.splitn(&subject, pieces).collect(),
        };

        lua.create_sequence_from(parts)
    })
}

#[cfg(test)]
mod tests {
    use super::super::fixture::eval;

    #[test]
    fn cuts_the_text_on_every_match() {
        assert_eq!(
            eval(r#"return table.concat(regex.split("a, b,c", ",\\s*"), "|")"#).unwrap(),
            "a|b|c"
        );
    }

    #[test]
    fn a_limit_keeps_the_rest_in_the_last_piece() {
        assert_eq!(
            eval(r#"return table.concat(regex.split("a, b,c", ",\\s*", 2), "|")"#).unwrap(),
            "a|b,c"
        );
    }
}
