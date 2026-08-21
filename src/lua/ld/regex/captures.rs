use mlua::{Lua, Value, Variadic};
use regex::Captures;

pub fn owned(captures: &Captures) -> Vec<Option<String>> {
    captures
        .iter()
        .map(|group| group.map(|group| group.as_str().to_string()))
        .collect()
}

pub fn values(lua: &Lua, groups: &[Option<String>]) -> mlua::Result<Variadic<Value>> {
    groups
        .iter()
        .map(|group| match group {
            Some(text) => lua.create_string(text).map(Value::String),
            None => Ok(Value::Nil),
        })
        .collect()
}
