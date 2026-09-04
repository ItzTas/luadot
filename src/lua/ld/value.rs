use std::path::PathBuf;

use mlua::Value;

use super::constants::API;
use super::parse::{external, lookup};

pub fn expected(namespace: &str, option: &str, kind: &str) -> mlua::Error {
    external(format!("`{API}.{namespace}.{option}` takes {kind}"))
}

pub fn path(namespace: &str, value: &Value, option: &str, kind: &str) -> mlua::Result<PathBuf> {
    let raw = text(namespace, value, option)?;
    if raw.trim().is_empty() {
        return Err(external(format!(
            "`{API}.{namespace}.{option}` takes {kind}, got an empty string"
        )));
    }

    Ok(PathBuf::from(raw))
}

pub fn choice<T: Copy>(
    namespace: &str,
    value: &Value,
    option: &str,
    entries: &[(&str, T)],
    field: &str,
) -> mlua::Result<T> {
    lookup(entries, &text(namespace, value, option)?, field)
}

pub fn text(namespace: &str, value: &Value, option: &str) -> mlua::Result<String> {
    match value {
        Value::String(text) => Ok(text.to_str()?.to_string()),
        _ => Err(expected(namespace, option, "a string")),
    }
}

pub fn flag(namespace: &str, value: &Value, option: &str) -> mlua::Result<bool> {
    match value {
        Value::Boolean(flag) => Ok(*flag),
        _ => Err(expected(namespace, option, "true or false")),
    }
}

pub fn span(namespace: &str, value: &Value, option: &str, kind: &str) -> mlua::Result<u64> {
    let raw = text(namespace, value, option)?;
    let Some(seconds) = crate::utils::seconds(&raw) else {
        return Err(external(format!(
            "`{API}.{namespace}.{option}` takes {kind}, got `{raw}`"
        )));
    };

    Ok(seconds)
}

pub fn count(namespace: &str, value: &Value, option: &str) -> mlua::Result<u32> {
    let whole = match value {
        Value::Integer(count) => *count,
        Value::Number(count) if count.fract() == 0.0 => *count as i64,
        _ => return Err(expected(namespace, option, "a whole number")),
    };

    u32::try_from(whole).map_err(|_| expected(namespace, option, "a whole number of zero or more"))
}

pub fn keys(namespace: &str, value: &Value, option: &str) -> mlua::Result<Vec<String>> {
    let keys = match value {
        Value::String(key) => vec![key.to_str()?.to_string()],
        Value::Table(list) => list
            .clone()
            .sequence_values::<String>()
            .collect::<mlua::Result<Vec<String>>>()
            .map_err(|_| expected(namespace, option, "a key or a list of keys"))?,
        _ => return Err(expected(namespace, option, "a key or a list of keys")),
    };

    if keys.is_empty() || keys.iter().any(|key| key.trim().is_empty()) {
        return Err(expected(namespace, option, "a key or a list of keys"));
    }

    Ok(keys)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lua::runtime::runtime;

    #[test]
    fn span_rejects_an_unknown_unit() {
        let lua = runtime().unwrap();
        let value = Value::String(lua.create_string("30").unwrap());

        let err = span("opt", &value, "backup_age", "a span like \"30d\"")
            .unwrap_err()
            .to_string();

        assert!(err.contains("`ld.opt.backup_age` takes a span like \"30d\""));
        assert!(err.contains("got `30`"));
    }

    #[test]
    fn count_rejects_a_non_integer() {
        for value in [Value::Number(1.5), Value::Boolean(true), Value::Nil] {
            let err = count("opt", &value, "backup_keep").unwrap_err().to_string();

            assert!(err.contains("`ld.opt.backup_keep` takes a whole number"));
        }
    }

    #[test]
    fn keys_reads_a_list_of_keys() {
        let lua = runtime().unwrap();
        let list = lua
            .create_sequence_from(["age1first", "age1second"])
            .unwrap();

        assert_eq!(
            keys("crypt.lock", &Value::Table(list), "recipients").unwrap(),
            ["age1first", "age1second"]
        );
    }
}
