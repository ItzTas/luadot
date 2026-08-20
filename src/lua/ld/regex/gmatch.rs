use std::sync::atomic::{AtomicUsize, Ordering};

use mlua::{Function, Lua, MultiValue, Value, Variadic};

use super::captures::{owned, values};
use super::constants::{GMATCH, PATTERN, TEXT};
use super::parse::{compile, text};

pub fn function(lua: &Lua) -> mlua::Result<Function> {
    lua.create_function(|lua, (subject, pattern): (Value, Value)| {
        let subject = text(&subject, GMATCH, TEXT)?;
        let pattern = text(&pattern, GMATCH, PATTERN)?;

        let regex = compile(&pattern, GMATCH)?;
        let found: Vec<Vec<Option<String>>> = regex
            .captures_iter(&subject)
            .map(|captures| owned(&captures))
            .collect();

        let next = AtomicUsize::new(0);
        lua.create_function(move |lua, _: MultiValue| {
            let index = next.fetch_add(1, Ordering::Relaxed);
            let Some(groups) = found.get(index) else {
                return Ok(Variadic::new());
            };

            values(lua, groups)
        })
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
    fn carries_the_groups_of_each_match() {
        assert_eq!(
            eval(
                r#"
                local found = {}
                for _, key, value in regex.gmatch("a=1, b=2", "(\\w)=(\\d)") do
                  found[#found + 1] = key .. value
                end
                return table.concat(found, "|")
                "#
            )
            .unwrap(),
            "a1|b2"
        );
    }
}
