use mlua::{Function, Lua, Value};

use super::constants::{ESCAPE, TEXT};
use super::parse::text;

pub fn function(lua: &Lua) -> mlua::Result<Function> {
    lua.create_function(|_, subject: Value| Ok(regex::escape(&text(&subject, ESCAPE, TEXT)?)))
}

#[cfg(test)]
mod tests {
    use super::super::table::table;
    use crate::lua::runtime::runtime;

    fn eval(source: &str) -> mlua::Result<String> {
        let lua = runtime().unwrap();
        lua.globals().set("regex", table(&lua).unwrap()).unwrap();

        lua.load(source).eval()
    }

    #[test]
    fn turns_a_literal_into_an_expression_matching_itself() {
        assert_eq!(
            eval(
                r#"
                local literal = regex.escape("v1.0+build")
                return regex.match("v1.0+build", literal)
                "#
            )
            .unwrap(),
            "v1.0+build"
        );
    }
}
