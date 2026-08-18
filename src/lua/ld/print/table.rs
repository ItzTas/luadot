use mlua::{Lua, Table};

use super::super::table::build;
use super::constants::FUNCTIONS;
use super::line;

pub fn table(lua: &Lua) -> mlua::Result<Table> {
    let print = build(lua, &FUNCTIONS)?;

    let meta = lua.create_table()?;
    meta.set("__call", line::function(lua)?)?;
    print.set_metatable(Some(meta))?;

    Ok(print)
}

#[cfg(test)]
mod tests {
    use super::table;
    use crate::lua::runtime::runtime;

    #[test]
    fn the_namespace_is_callable_and_carries_every_call() {
        let lua = runtime().unwrap();
        lua.globals().set("print_", table(&lua).unwrap()).unwrap();

        lua.load(
            r#"
            assert(type(getmetatable(print_).__call) == "function", "print is not callable")
            for _, name in ipairs({ "note", "warn", "error", "section", "entry", "field" }) do
              assert(type(print_[name]) == "function", "print." .. name .. " is missing")
            end
            "#,
        )
        .exec()
        .unwrap();
    }
}
