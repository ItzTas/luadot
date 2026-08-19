use mlua::Value;
use regex::Regex;

use super::super::constants::API;
use super::super::parse::external;
use super::constants::NAMESPACE;

pub fn prefix(function: &str) -> String {
    format!("`{API}.{NAMESPACE}.{function}`")
}

pub fn text(value: &Value, function: &str, argument: &str) -> mlua::Result<String> {
    match value {
        Value::String(text) => Ok(text.to_str()?.to_string()),
        _ => Err(external(format!(
            "{} takes {argument} as a string",
            prefix(function)
        ))),
    }
}

pub fn compile(raw: &str, function: &str) -> mlua::Result<Regex> {
    Regex::new(raw).map_err(|err| {
        external(format!(
            "{}: invalid regex `{raw}`: {err}",
            prefix(function)
        ))
    })
}

pub fn limit(value: &Value, function: &str) -> mlua::Result<usize> {
    let whole = match value {
        Value::Nil => return Ok(0),
        Value::Integer(limit) => *limit,
        Value::Number(limit) if limit.fract() == 0.0 => *limit as i64,
        _ => return Err(broken(function)),
    };

    usize::try_from(whole).map_err(|_| broken(function))
}

fn broken(function: &str) -> mlua::Error {
    external(format!(
        "{} takes the limit as a whole number of zero or more",
        prefix(function)
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lua::runtime::runtime;

    #[test]
    fn text_reads_a_string() {
        let lua = runtime().unwrap();
        let value = Value::String(lua.create_string("one").unwrap());

        assert_eq!(text(&value, "gsub", "the text").unwrap(), "one");
    }

    #[test]
    fn text_names_the_call_and_the_argument_it_wanted() {
        let err = text(&Value::Boolean(true), "gsub", "the pattern")
            .unwrap_err()
            .to_string();

        assert!(err.contains("`ld.regex.gsub` takes the pattern as a string"));
    }

    #[test]
    fn compile_reports_an_invalid_expression() {
        let err = compile("^[", "match").unwrap_err().to_string();

        assert!(err.contains("`ld.regex.match`: invalid regex `^[`"));
    }

    #[test]
    fn limit_defaults_to_every_occurrence() {
        assert_eq!(limit(&Value::Nil, "gsub").unwrap(), 0);
        assert_eq!(limit(&Value::Integer(2), "gsub").unwrap(), 2);
        assert_eq!(limit(&Value::Number(2.0), "gsub").unwrap(), 2);
    }

    #[test]
    fn limit_rejects_a_fraction_a_negative_number_and_anything_else() {
        for value in [Value::Number(1.5), Value::Integer(-1), Value::Boolean(true)] {
            let err = limit(&value, "gsub").unwrap_err().to_string();

            assert!(
                err.contains("`ld.regex.gsub` takes the limit as a whole number of zero or more")
            );
        }
    }
}
