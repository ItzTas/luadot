use mlua::{Lua, Table};

use super::{program, shell};

pub fn table(lua: &Lua) -> mlua::Result<Table> {
    let cmd = lua.create_table()?;

    let meta = lua.create_table()?;
    meta.set("__call", shell::function(lua)?)?;
    meta.set("__index", program::function(lua)?)?;
    cmd.set_metatable(Some(meta))?;

    Ok(cmd)
}

#[cfg(test)]
mod tests {
    use super::table;
    use crate::lua::runtime::runtime;

    #[test]
    fn the_namespace_is_callable_and_indexable() {
        let lua = runtime().unwrap();
        lua.globals().set("cmd", table(&lua).unwrap()).unwrap();

        lua.load(
            r#"
            local meta = getmetatable(cmd)
            assert(type(meta.__call) == "function", "cmd is not callable")
            assert(type(meta.__index) == "function", "cmd is not indexable")
            assert(type(cmd.ls) == "function", "cmd.ls is not a function")
            "#,
        )
        .exec()
        .unwrap();
    }
}
