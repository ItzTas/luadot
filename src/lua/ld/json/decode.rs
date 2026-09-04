use mlua::{Function, Lua, LuaSerdeExt, Value};
use serde_json::Value as Json;

use super::super::constants::API;
use super::super::parse::external;
use super::constants::{DECODE, NAMESPACE};

pub fn function(lua: &Lua) -> mlua::Result<Function> {
    lua.create_function(|lua, text: String| decode(lua, &text))
}

fn decode(lua: &Lua, text: &str) -> mlua::Result<Value> {
    let json: Json = serde_json::from_str(text).map_err(|err| {
        external(format!(
            "`{API}.{NAMESPACE}.{DECODE}` failed to parse: {err}"
        ))
    })?;

    lua.to_value(&json)
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
    fn nesting_past_what_the_parser_takes_is_refused() {
        let deep = format!("{}1{}", "[".repeat(200), "]".repeat(200));

        let err = eval(&format!(r#"return json.decode("{deep}")"#))
            .unwrap_err()
            .to_string();

        assert!(err.contains("`ld.json.decode` failed to parse"));
    }
}
