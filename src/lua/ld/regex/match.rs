use mlua::{Function, Lua, Value, Variadic};

use super::captures::{owned, values};
use super::constants::{MATCH, PATTERN, TEXT};
use super::parse::{compile, text};

pub fn function(lua: &Lua) -> mlua::Result<Function> {
    lua.create_function(|lua, (subject, pattern): (Value, Value)| {
        let subject = text(&subject, MATCH, TEXT)?;
        let pattern = text(&pattern, MATCH, PATTERN)?;

        let regex = compile(&pattern, MATCH)?;
        let Some(captures) = regex.captures(&subject) else {
            return Ok(Variadic::new());
        };

        values(lua, &owned(&captures))
    })
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
    fn yields_the_whole_match_before_its_groups() {
        assert_eq!(
            eval(
                r#"
                local whole, name, version = regex.match("neovim@0.11.2", "(\\w+)@([\\d.]+)")
                return whole .. "|" .. name .. "|" .. version
                "#
            )
            .unwrap(),
            "neovim@0.11.2|neovim|0.11.2"
        );
    }

    #[test]
    fn yields_nothing_without_a_match() {
        assert_eq!(
            eval(r#"return regex.match("neovim", "\\d+") == nil and "nil" or "value""#).unwrap(),
            "nil"
        );
    }

    #[test]
    fn reports_an_invalid_expression() {
        let err = eval(r#"return regex.match("neovim", "(")"#)
            .unwrap_err()
            .to_string();

        assert!(err.contains("`ld.regex.match`: invalid regex `(`"));
    }

    #[test]
    fn reports_an_argument_that_is_not_a_string() {
        let err = eval(r#"return regex.match({}, "a")"#)
            .unwrap_err()
            .to_string();

        assert!(err.contains("`ld.regex.match` takes the text as a string"));
    }
}
