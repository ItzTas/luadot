use mlua::{Function, Lua, Table, Value};

use super::constants::{INDEX, LPEG_MODULE, RE_MODULE, REQUIRE};

pub fn install(lua: &Lua, ld: &Table) -> mlua::Result<()> {
    let meta = lua.create_table()?;
    meta.set(INDEX, lua.create_function(resolve)?)?;
    ld.set_metatable(Some(meta))
}

fn resolve(lua: &Lua, (ld, key): (Table, String)) -> mlua::Result<Value> {
    if key != LPEG_MODULE && key != RE_MODULE {
        return Ok(Value::Nil);
    }

    let require: Function = lua.globals().get(REQUIRE)?;
    let module: Value = require.call(key.clone())?;
    ld.raw_set(key, module.clone())?;

    Ok(module)
}
