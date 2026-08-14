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

pub fn flag(value: &Value, option: &str) -> mlua::Result<bool> {
    match value {
        Value::Boolean(flag) => Ok(*flag),
        _ => Err(expected(option, "true or false")),
    }
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
        let value = Value::String(lua.create_string("hard").unwrap());

        assert_eq!(text(&value, "link").unwrap(), "hard");
    }

    #[test]
    fn text_rejects_anything_else() {
        let err = text(&Value::Boolean(true), "link").unwrap_err().to_string();

        assert!(err.contains("`ld.opt.link` takes a string"));
    }

    #[test]
    fn flag_reads_a_boolean() {
        assert!(!flag(&Value::Boolean(false), "pkg_warn").unwrap());
        assert!(flag(&Value::Boolean(true), "pkg_warn").unwrap());
    }

    #[test]
    fn flag_rejects_anything_else() {
        let err = flag(&Value::Integer(0), "pkg_warn")
            .unwrap_err()
            .to_string();

        assert!(err.contains("`ld.opt.pkg_warn` takes true or false"));
    }
}
