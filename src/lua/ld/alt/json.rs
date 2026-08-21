use mlua::{Function, Lua, Table, Value};
use serde_json::{Map, Number, Value as Json};

use super::super::constants::API;
use super::super::parse::external;
use super::constants::{JSON, JSON_DEPTH, NAMESPACE};

pub fn function(lua: &Lua) -> mlua::Result<Function> {
    lua.create_function(|_, value: Value| {
        let json = convert(&value, 0)?;

        serde_json::to_string_pretty(&json).map_err(|err| {
            external(format!(
                "`{API}.{NAMESPACE}.{JSON}` failed to serialize: {err}"
            ))
        })
    })
}

fn convert(value: &Value, depth: usize) -> mlua::Result<Json> {
    if depth > JSON_DEPTH {
        return Err(external(format!(
            "`{API}.{NAMESPACE}.{JSON}` gave up below {JSON_DEPTH} nested tables; a table holding itself never ends"
        )));
    }

    match value {
        Value::Nil => Ok(Json::Null),
        Value::Boolean(state) => Ok(Json::Bool(*state)),
        Value::Integer(number) => Ok(Json::from(*number)),
        Value::Number(number) => float(*number),
        Value::String(text) => Ok(Json::String(text.to_str()?.to_string())),
        Value::Table(table) => collect(table, depth),
        other => Err(external(format!(
            "`{API}.{NAMESPACE}.{JSON}` cannot serialize {}",
            other.type_name()
        ))),
    }
}

fn float(number: f64) -> mlua::Result<Json> {
    Number::from_f64(number).map(Json::Number).ok_or_else(|| {
        external(format!(
            "`{API}.{NAMESPACE}.{JSON}` cannot serialize {number}"
        ))
    })
}

fn collect(table: &Table, depth: usize) -> mlua::Result<Json> {
    let length = table.raw_len();
    let entries = table.pairs::<Value, Value>().count();

    if length == 0 {
        return object(table, depth);
    }
    if length == entries {
        return array(table, depth);
    }

    Err(external(format!(
        "`{API}.{NAMESPACE}.{JSON}` got a table mixing a list of {length} value(s) with named keys"
    )))
}

fn array(table: &Table, depth: usize) -> mlua::Result<Json> {
    let mut values = Vec::new();
    for value in table.sequence_values::<Value>() {
        values.push(convert(&value?, depth + 1)?);
    }

    Ok(Json::Array(values))
}

fn object(table: &Table, depth: usize) -> mlua::Result<Json> {
    let mut entries = Map::new();
    for pair in table.pairs::<Value, Value>() {
        let (key, value) = pair?;
        entries.insert(name(&key)?, convert(&value, depth + 1)?);
    }

    Ok(Json::Object(entries))
}

fn name(key: &Value) -> mlua::Result<String> {
    match key {
        Value::String(text) => Ok(text.to_str()?.to_string()),
        other => Err(external(format!(
            "`{API}.{NAMESPACE}.{JSON}` needs string keys, got a {} one",
            other.type_name()
        ))),
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::lua::ld::{Paths, Surface, install};
    use crate::lua::runtime::runtime;
    use crate::state::Classes;

    fn json(source: &str) -> mlua::Result<String> {
        let lua = runtime().unwrap();
        let paths = Paths::new(Path::new("/home/u"), Path::new("/home/u/.config/luadot"));
        install(&lua, Surface::Standalone, &paths, &Classes::default()).unwrap();

        lua.load(source).eval()
    }

    fn error(source: &str) -> String {
        json(source).unwrap_err().to_string()
    }

    #[test]
    fn a_table_of_names_becomes_an_object_with_sorted_keys() {
        assert_eq!(
            json(r#"return ld.alt.json({ editor = "nvim", gpu = "amd" })"#).unwrap(),
            "{\n  \"editor\": \"nvim\",\n  \"gpu\": \"amd\"\n}"
        );
    }

    #[test]
    fn every_scalar_keeps_its_type() {
        assert_eq!(json("return ld.alt.json(true)").unwrap(), "true");
        assert_eq!(json("return ld.alt.json(2)").unwrap(), "2");
        assert_eq!(json("return ld.alt.json(2.5)").unwrap(), "2.5");
        assert_eq!(json(r#"return ld.alt.json("text")"#).unwrap(), "\"text\"");
        assert_eq!(json("return ld.alt.json(nil)").unwrap(), "null");
    }

    #[test]
    fn a_table_holding_itself_is_reported() {
        let err = error("local t = {}; t.self = t; return ld.alt.json(t)");

        assert!(err.contains("a table holding itself never ends"));
    }
}
