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

pub fn count(value: &Value, option: &str) -> mlua::Result<u32> {
    let whole = match value {
        Value::Integer(count) => *count,
        Value::Number(count) if count.fract() == 0.0 => *count as i64,
        _ => return Err(expected(option, "a whole number")),
    };

    u32::try_from(whole).map_err(|_| expected(option, "a whole number of zero or more"))
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

    #[test]
    fn count_reads_a_whole_number() {
        assert_eq!(count(&Value::Integer(5), "backup_keep").unwrap(), 5);
        assert_eq!(count(&Value::Number(5.0), "backup_keep").unwrap(), 5);
        assert_eq!(count(&Value::Integer(0), "backup_keep").unwrap(), 0);
    }

    #[test]
    fn count_rejects_a_fraction_and_anything_that_is_not_a_number() {
        for value in [Value::Number(1.5), Value::Boolean(true), Value::Nil] {
            let err = count(&value, "backup_keep").unwrap_err().to_string();

            assert!(err.contains("`ld.opt.backup_keep` takes a whole number"));
        }
    }

    #[test]
    fn count_rejects_a_negative_number() {
        let err = count(&Value::Integer(-1), "backup_keep")
            .unwrap_err()
            .to_string();

        assert!(err.contains("takes a whole number of zero or more"));
    }
}
