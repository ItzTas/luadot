use mlua::{Lua, Table};

use crate::lua::runtime::runtime;

pub fn eval(
    namespace: &str,
    build: impl FnOnce(&Lua) -> mlua::Result<Table>,
    source: &str,
) -> mlua::Result<String> {
    let lua = runtime().unwrap();
    lua.globals().set(namespace, build(&lua).unwrap()).unwrap();

    lua.load(source).eval()
}
