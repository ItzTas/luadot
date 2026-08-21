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
