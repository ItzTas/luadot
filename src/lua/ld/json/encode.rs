use mlua::{Function, Lua, LuaSerdeExt, Table, Value};
use serde_json::Value as Json;

use super::super::constants::API;
use super::super::parse::external;
use super::constants::{DEPTH, ENCODE, NAMESPACE};

pub fn function(lua: &Lua) -> mlua::Result<Function> {
    encoder(lua, &format!("{NAMESPACE}.{ENCODE}"))
}

pub fn encoder(lua: &Lua, call: &str) -> mlua::Result<Function> {
    let call = call.to_string();

    lua.create_function(move |lua, value: Value| encode(lua, &call, value))
}

fn encode(lua: &Lua, call: &str, value: Value) -> mlua::Result<String> {
    shaped(call, &value, 0)?;

    let json: Json = lua
        .from_value(value)
        .map_err(|err| external(format!("`{API}.{call}` cannot serialize: {err}")))?;

    serde_json::to_string_pretty(&json)
        .map_err(|err| external(format!("`{API}.{call}` failed to serialize: {err}")))
}

fn shaped(call: &str, value: &Value, depth: usize) -> mlua::Result<()> {
    if let Value::Number(number) = value
        && !number.is_finite()
    {
        return Err(external(format!(
            "`{API}.{call}` cannot serialize {number}"
        )));
    }

    let Value::Table(table) = value else {
        return Ok(());
    };
    if depth > DEPTH {
        return Err(external(format!(
            "`{API}.{call}` gave up below {DEPTH} nested tables; a table holding itself never ends"
        )));
    }

    listed(call, table)?;
    for pair in table.pairs::<Value, Value>() {
        shaped(call, &pair?.1, depth + 1)?;
    }

    Ok(())
}

fn listed(call: &str, table: &Table) -> mlua::Result<()> {
    let length = table.raw_len();
    if length == 0 || length == table.pairs::<Value, Value>().count() {
        return Ok(());
    }

    Err(external(format!(
        "`{API}.{call}` got a table mixing a list of {length} value(s) with named keys"
    )))
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
    fn a_table_of_names_keeps_its_keys_sorted() {
        assert_eq!(
            eval(r#"return json.encode({ b = 2, a = 1 })"#).unwrap(),
            "{\n  \"a\": 1,\n  \"b\": 2\n}"
        );
    }

    #[test]
    fn an_empty_table_encodes_as_a_table_of_names() {
        assert_eq!(eval("return json.encode({})").unwrap(), "{}");
    }

    #[test]
    fn a_table_mixing_a_list_with_names_is_refused() {
        let err = eval("return json.encode({ 1, name = true })")
            .unwrap_err()
            .to_string();

        assert!(err.contains("got a table mixing a list of 1 value(s) with named keys"));
    }

    #[test]
    fn the_message_names_the_call_that_ran() {
        let err = eval("return json.encode({ f = print })")
            .unwrap_err()
            .to_string();

        assert!(err.contains("`ld.json.encode` cannot serialize"));
        assert!(err.contains("function"));
    }
}
