use mlua::{Lua, Table};

use super::constants::GET;
use super::{declare, get};
use crate::state::Classes;

pub fn table(lua: &Lua, classes: &Classes) -> mlua::Result<Table> {
    let class = lua.create_table()?;
    class.set(GET, get::function(lua, classes)?)?;

    let meta = lua.create_table()?;
    meta.set("__call", declare::function(lua)?)?;
    class.set_metatable(Some(meta))?;

    Ok(class)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lua::runtime::runtime;

    #[test]
    fn the_namespace_declares_and_reads() {
        let lua = runtime().unwrap();

        let class = table(&lua, &Classes::default()).unwrap();

        assert!(class.get::<mlua::Function>(GET).is_ok());
        assert!(
            class
                .metatable()
                .unwrap()
                .get::<mlua::Function>("__call")
                .is_ok()
        );
    }
}
