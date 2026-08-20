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
    fn text_rejects_anything_else() {
        let err = text("opt", &Value::Boolean(true), "link")
            .unwrap_err()
            .to_string();

        assert!(err.contains("`ld.opt.link` takes a string"));
    }

    #[test]
    fn text_names_the_namespace_it_was_given() {
        let err = text("crypt", &Value::Boolean(true), "backend")
            .unwrap_err()
            .to_string();

        assert!(err.contains("`ld.crypt.backend` takes a string"));
    }

    #[test]
    fn flag_rejects_anything_else() {
        let err = flag("opt", &Value::Integer(0), "pkg_warn")
            .unwrap_err()
            .to_string();

        assert!(err.contains("`ld.opt.pkg_warn` takes true or false"));
    }

    #[test]
    fn span_rejects_a_string_without_a_known_unit() {
        let lua = runtime().unwrap();
        let value = Value::String(lua.create_string("30").unwrap());

        let err = span("opt", &value, "backup_age", "a span like \"30d\"")
            .unwrap_err()
            .to_string();

        assert!(err.contains("`ld.opt.backup_age` takes a span like \"30d\""));
        assert!(err.contains("got `30`"));
    }

    #[test]
    fn count_rejects_a_fraction_and_anything_that_is_not_a_number() {
        for value in [Value::Number(1.5), Value::Boolean(true), Value::Nil] {
            let err = count("opt", &value, "backup_keep").unwrap_err().to_string();

            assert!(err.contains("`ld.opt.backup_keep` takes a whole number"));
        }
    }

    #[test]
    fn keys_reads_a_single_key() {
        let lua = runtime().unwrap();
        let value = Value::String(lua.create_string("age1example").unwrap());

        assert_eq!(
            keys("crypt.lock", &value, "recipients").unwrap(),
            ["age1example"]
        );
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

    #[test]
    fn keys_rejects_an_empty_list() {
        let lua = runtime().unwrap();
        let list = lua.create_table().unwrap();

        let err = keys("crypt.lock", &Value::Table(list), "recipients")
            .unwrap_err()
            .to_string();

        assert!(err.contains("`ld.crypt.lock.recipients` takes a key or a list of keys"));
    }

    #[test]
    fn keys_rejects_anything_else() {
        assert!(keys("crypt.lock", &Value::Integer(1), "recipients").is_err());
    }
}
