use mlua::{Lua, Table};

use super::super::table::build;
use super::constants::FUNCTIONS;

pub fn table(lua: &Lua) -> mlua::Result<Table> {
    build(lua, &FUNCTIONS)
}

#[cfg(test)]
mod tests {
    use super::table;
    use crate::lua::runtime::runtime;

    #[test]
    fn the_namespace_carries_a_call_per_command() {
        let lua = runtime().unwrap();
        lua.globals().set("on", table(&lua).unwrap()).unwrap();

        lua.load(
            r#"
            for _, name in ipairs({ "diff", "status" }) do
              assert(type(on[name]) == "function", "on." .. name .. " is missing")
            end
            "#,
        )
        .exec()
        .unwrap();
    }
}
