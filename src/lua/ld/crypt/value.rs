use mlua::Value;

use super::super::constants::API;
use super::super::parse::external;
use super::constants::NAMESPACE;

pub fn text(value: &Value, option: &str) -> mlua::Result<String> {
    match value {
        Value::String(text) => Ok(text.to_str()?.to_string()),
        _ => Err(expected(option, "a string")),
    }
}

pub fn keys(value: &Value, option: &str) -> mlua::Result<Vec<String>> {
    let keys = match value {
        Value::String(key) => vec![key.to_str()?.to_string()],
        Value::Table(list) => list
            .clone()
            .sequence_values::<String>()
            .collect::<mlua::Result<Vec<String>>>()
            .map_err(|_| expected(option, "a key or a list of keys"))?,
        _ => return Err(expected(option, "a key or a list of keys")),
    };

    if keys.is_empty() || keys.iter().any(|key| key.trim().is_empty()) {
        return Err(expected(option, "a key or a list of keys"));
    }

    Ok(keys)
}

fn expected(option: &str, kind: &str) -> mlua::Error {
    external(format!("`{API}.{NAMESPACE}.{option}` takes {kind}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lua::runtime::runtime;

    #[test]
    fn text_reads_a_string() {
        let lua = runtime().unwrap();
        let value = Value::String(lua.create_string("age").unwrap());

        assert_eq!(text(&value, "backend").unwrap(), "age");
    }

    #[test]
    fn text_rejects_anything_else() {
        let err = text(&Value::Boolean(true), "backend")
            .unwrap_err()
            .to_string();

        assert!(err.contains("`ld.crypt.backend` takes a string"));
    }

    #[test]
    fn keys_reads_a_single_key() {
        let lua = runtime().unwrap();
        let value = Value::String(lua.create_string("age1example").unwrap());

        assert_eq!(keys(&value, "recipients").unwrap(), ["age1example"]);
    }

    #[test]
    fn keys_reads_a_list_of_keys() {
        let lua = runtime().unwrap();
        let list = lua
            .create_sequence_from(["age1first", "age1second"])
            .unwrap();

        assert_eq!(
            keys(&Value::Table(list), "recipients").unwrap(),
            ["age1first", "age1second"]
        );
    }

    #[test]
    fn keys_rejects_an_empty_list() {
        let lua = runtime().unwrap();
        let list = lua.create_table().unwrap();

        let err = keys(&Value::Table(list), "recipients")
            .unwrap_err()
            .to_string();

        assert!(err.contains("`ld.crypt.recipients` takes a key or a list of keys"));
    }

    #[test]
    fn keys_rejects_a_blank_key() {
        let lua = runtime().unwrap();
        let value = Value::String(lua.create_string("  ").unwrap());

        assert!(keys(&value, "recipients").is_err());
    }

    #[test]
    fn keys_rejects_anything_else() {
        assert!(keys(&Value::Integer(1), "recipients").is_err());
    }
}
