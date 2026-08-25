use mlua::{Function, Lua, Value};

use super::super::constants::API;
use super::super::parse::external;
use super::constants::{GET, NAMESPACE};
use super::values::current;

pub fn function(lua: &Lua) -> mlua::Result<Function> {
    lua.create_function(move |lua, value: Value| {
        let name = name(&value)?;

        Ok(current(lua).get(&name).map(str::to_string))
    })
}

fn name(value: &Value) -> mlua::Result<String> {
    match value {
        Value::String(name) => Ok(name.to_str()?.to_string()),
        other => Err(external(format!(
            "`{API}.{NAMESPACE}.{GET}` takes a class name, got {}",
            other.type_name()
        ))),
    }
}
