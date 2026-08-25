use mlua::{Function, Lua, Table, Value};
use serde_json::{Map, Number, Value as Json};

use super::super::constants::API;
use super::super::parse::external;
use super::constants::{DEPTH, ENCODE, NAMESPACE};
use super::null::is_null;

pub fn function(lua: &Lua) -> mlua::Result<Function> {
    encoder(lua, &format!("{NAMESPACE}.{ENCODE}"))
}

pub fn encoder(lua: &Lua, call: &str) -> mlua::Result<Function> {
    let call = call.to_string();

    lua.create_function(move |_, value: Value| encode(&call, &value))
}

fn encode(call: &str, value: &Value) -> mlua::Result<String> {
    let json = convert(call, value, 0)?;

    serde_json::to_string_pretty(&json)
        .map_err(|err| external(format!("`{API}.{call}` failed to serialize: {err}")))
}

fn convert(call: &str, value: &Value, depth: usize) -> mlua::Result<Json> {
    if depth > DEPTH {
        return Err(external(format!(
            "`{API}.{call}` gave up below {DEPTH} nested tables; a table holding itself never ends"
        )));
    }
    if is_null(value) {
        return Ok(Json::Null);
    }

    match value {
        Value::Nil => Ok(Json::Null),
        Value::Boolean(state) => Ok(Json::Bool(*state)),
        Value::Integer(number) => Ok(Json::from(*number)),
        Value::Number(number) => float(call, *number),
        Value::String(text) => Ok(Json::String(text.to_str()?.to_string())),
        Value::Table(table) => collect(call, table, depth),
        other => Err(external(format!(
            "`{API}.{call}` cannot serialize {}",
            other.type_name()
        ))),
    }
}

fn float(call: &str, number: f64) -> mlua::Result<Json> {
    Number::from_f64(number)
        .map(Json::Number)
        .ok_or_else(|| external(format!("`{API}.{call}` cannot serialize {number}")))
}

fn collect(call: &str, table: &Table, depth: usize) -> mlua::Result<Json> {
    let length = table.raw_len();
    let entries = table.pairs::<Value, Value>().count();

    if length == 0 {
        return object(call, table, depth);
    }
    if length == entries {
        return array(call, table, depth);
    }

    Err(external(format!(
        "`{API}.{call}` got a table mixing a list of {length} value(s) with named keys"
    )))
}

fn array(call: &str, table: &Table, depth: usize) -> mlua::Result<Json> {
    let mut values = Vec::new();
    for value in table.sequence_values::<Value>() {
        values.push(convert(call, &value?, depth + 1)?);
    }

    Ok(Json::Array(values))
}

fn object(call: &str, table: &Table, depth: usize) -> mlua::Result<Json> {
    let mut entries = Map::new();
    for pair in table.pairs::<Value, Value>() {
        let (key, value) = pair?;
        entries.insert(name(call, &key)?, convert(call, &value, depth + 1)?);
    }

    Ok(Json::Object(entries))
}

fn name(call: &str, key: &Value) -> mlua::Result<String> {
    match key {
        Value::String(text) => Ok(text.to_str()?.to_string()),
        other => Err(external(format!(
            "`{API}.{call}` needs string keys, got a {} one",
            other.type_name()
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::super::fixture::eval;

    #[test]
    fn null_encodes_inside_a_table() {
        assert_eq!(
            eval("return json.encode({ json.null, 1 })").unwrap(),
            "[\n  null,\n  1\n]"
        );
        assert_eq!(
            eval("return json.encode({ gone = json.null })").unwrap(),
            "{\n  \"gone\": null\n}"
        );
    }

    #[test]
    fn the_message_names_the_call_that_ran() {
        let err = eval("return json.encode({ f = print })")
            .unwrap_err()
            .to_string();

        assert!(err.contains("`ld.json.encode` cannot serialize function"));
    }
}
