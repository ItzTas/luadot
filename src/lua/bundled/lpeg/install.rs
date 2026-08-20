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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lua::runtime::runtime;

    #[test]
    fn the_modules_answer_from_the_api_and_stay_the_same_tables() {
        let lua = runtime().unwrap();
        let ld = lua.create_table().unwrap();
        install(&lua, &ld).unwrap();
        lua.globals().set("ld", ld).unwrap();

        lua.load(
            r#"
            assert(ld.lpeg == require("lpeg"), "ld.lpeg is not the lpeg module")
            assert(ld.re == require("re"), "ld.re is not the re module")
            assert(rawget(ld, "lpeg") ~= nil, "ld.lpeg was not cached")
            assert(ld.nothing == nil, "an unknown field answered something")
            "#,
        )
        .exec()
        .unwrap();
    }
}
