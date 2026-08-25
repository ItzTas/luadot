use mlua::{Function, Lua, Value};
use serde_json::{Number, Value as Json};

use super::super::constants::API;
use super::super::parse::external;
use super::constants::{DECODE, DEPTH, NAMESPACE};
use super::null;

pub fn function(lua: &Lua) -> mlua::Result<Function> {
    lua.create_function(|lua, text: String| decode(lua, &text))
}

fn decode(lua: &Lua, text: &str) -> mlua::Result<Value> {
    let json: Json = serde_json::from_str(text)
        .map_err(|err| external(format!("{} failed to parse: {err}", prefix())))?;

    convert(lua, &json, 0)
}

fn convert(lua: &Lua, json: &Json, depth: usize) -> mlua::Result<Value> {
    if depth > DEPTH {
        return Err(external(format!(
            "{} gave up below {DEPTH} nested values",
            prefix()
        )));
    }

    match json {
        Json::Null => Ok(null::value()),
        Json::Bool(state) => Ok(Value::Boolean(*state)),
        Json::Number(number) => self::number(number),
        Json::String(text) => Ok(Value::String(lua.create_string(text)?)),
        Json::Array(items) => {
            let table = lua.create_table_with_capacity(items.len(), 0)?;
            for (index, item) in items.iter().enumerate() {
                table.raw_set(index + 1, convert(lua, item, depth + 1)?)?;
            }
            Ok(Value::Table(table))
        }
        Json::Object(entries) => {
            let table = lua.create_table_with_capacity(0, entries.len())?;
            for (key, value) in entries {
                table.raw_set(key.as_str(), convert(lua, value, depth + 1)?)?;
            }
            Ok(Value::Table(table))
        }
    }
}

fn number(number: &Number) -> mlua::Result<Value> {
    if let Some(whole) = number.as_i64() {
        return Ok(Value::Integer(whole));
    }

    number
        .as_f64()
        .map(Value::Number)
        .ok_or_else(|| external(format!("{} cannot hold {number}", prefix())))
}

fn prefix() -> String {
    format!("`{API}.{NAMESPACE}.{DECODE}`")
}

#[cfg(test)]
mod tests {
    use super::super::fixture::eval;

    #[test]
    fn objects_and_lists_become_tables() {
        assert_eq!(
            eval(
                r#"
                local lock = json.decode('{ "plugins": [ { "name": "a", "rev": "abc" } ], "v": 2 }')
                return lock.plugins[1].name .. "@" .. lock.plugins[1].rev .. " v" .. lock.v
                "#
            )
            .unwrap(),
            "a@abc v2"
        );
    }

    #[test]
    fn null_keeps_its_place_in_a_list() {
        assert_eq!(
            eval(
                r#"
                local list = json.decode("[1, null, 3]")
                return #list .. "/" .. tostring(list[2] == json.null)
                "#
            )
            .unwrap(),
            "3/true"
        );
    }

    #[test]
    fn whole_numbers_are_integers() {
        assert_eq!(
            eval(r#"return math.type(json.decode("2")) .. "/" .. math.type(json.decode("2.5"))"#)
                .unwrap(),
            "integer/float"
        );
    }

    #[test]
    fn a_text_that_is_not_json_is_reported() {
        let err = eval(r#"return json.decode("{ oops }")"#)
            .unwrap_err()
            .to_string();

        assert!(err.contains("`ld.json.decode` failed to parse"));
    }

    #[test]
    fn nesting_past_the_limit_is_refused() {
        let deep = format!("{}1{}", "[".repeat(70), "]".repeat(70));

        let err = eval(&format!(r#"return json.decode("{deep}")"#))
            .unwrap_err()
            .to_string();

        assert!(err.contains("gave up below 64 nested values"));
    }
}
