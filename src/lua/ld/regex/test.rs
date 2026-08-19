use mlua::{Function, Lua, Value};

use super::constants::{PATTERN, TEST, TEXT};
use super::parse::{compile, text};

pub fn function(lua: &Lua) -> mlua::Result<Function> {
    lua.create_function(|_, (subject, pattern): (Value, Value)| {
        let subject = text(&subject, TEST, TEXT)?;
        let pattern = text(&pattern, TEST, PATTERN)?;

        Ok(compile(&pattern, TEST)?.is_match(&subject))
    })
}

#[cfg(test)]
mod tests {
    use super::super::table::table;
    use crate::lua::runtime::runtime;

    fn eval(source: &str) -> mlua::Result<bool> {
        let lua = runtime().unwrap();
        lua.globals().set("regex", table(&lua).unwrap()).unwrap();

        lua.load(source).eval()
    }

    #[test]
    fn answers_whether_the_expression_is_anywhere_in_the_text() {
        assert!(eval(r#"return regex.test("nvim 0.11.2", "\\d+\\.\\d+")"#).unwrap());
        assert!(!eval(r#"return regex.test("nvim", "\\d+")"#).unwrap());
    }
}
