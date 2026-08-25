use mlua::Lua;

use crate::state::Classes;

pub fn install(lua: &Lua, classes: &Classes) {
    lua.set_app_data(classes.clone());
}

pub fn remember(lua: &Lua, name: &str, value: &str) {
    let mut classes = current(lua);
    classes.set(name, value);
    install(lua, &classes);
}

pub fn current(lua: &Lua) -> Classes {
    lua.app_data_ref::<Classes>()
        .map(|classes| classes.clone())
        .unwrap_or_default()
}
