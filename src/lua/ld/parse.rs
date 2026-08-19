use glob::Pattern;
use mlua::{Table, Value};
use regex::Regex;

use super::constants::{CONFLICT_POLICIES, LINK_MODES, MATCH, REGEX};
use crate::files::{ConflictPolicy, LinkMode};
use crate::lua::Matcher;

pub fn external(message: impl Into<String>) -> mlua::Error {
    mlua::Error::external(message.into())
}

pub fn chain(err: anyhow::Error) -> mlua::Error {
    external(format!("{err:#}"))
}

pub fn matcher(entry: &Table) -> mlua::Result<Matcher> {
    let glob: Option<Value> = entry.get(MATCH)?;
    let expression: Option<Value> = entry.get(REGEX)?;

    match (glob, expression) {
        (Some(_), Some(_)) => Err(external(format!(
            "a rule takes `{MATCH}` or `{REGEX}`, not both"
        ))),
        (Some(glob), None) => alternatives(&glob, MATCH, pattern),
        (None, Some(expression)) => alternatives(&expression, REGEX, regex),
        (None, None) => Err(external(format!(
            "a rule needs a `{MATCH}` or `{REGEX}` pattern"
        ))),
    }
}

fn alternatives(
    value: &Value,
    field: &str,
    build: fn(&str) -> mlua::Result<Matcher>,
) -> mlua::Result<Matcher> {
    if let Some(raw) = value.as_string() {
        return build(&raw.to_str()?);
    }

    let list = value
        .as_table()
        .ok_or_else(|| external(format!("`{field}` takes a string or a table of strings")))?;

    let mut matchers = list
        .clone()
        .sequence_values::<String>()
        .enumerate()
        .map(|(index, raw)| {
            let raw = raw
                .map_err(|_| external(format!("`{field}` entry {} is not a string", index + 1)))?;
            build(&raw)
        })
        .collect::<mlua::Result<Vec<_>>>()?;

    if matchers.is_empty() {
        return Err(external(format!("`{field}` needs at least one pattern")));
    }
    if matchers.len() == 1 {
        return Ok(matchers.remove(0));
    }

    Ok(Matcher::Any(matchers))
}

pub fn pattern(raw: &str) -> mlua::Result<Matcher> {
    Pattern::new(raw)
        .map(Matcher::Glob)
        .map_err(|err| external(format!("invalid pattern `{raw}`: {err}")))
}

pub fn regex(raw: &str) -> mlua::Result<Matcher> {
    Regex::new(raw)
        .map(Matcher::Regex)
        .map_err(|err| external(format!("invalid regex `{raw}`: {err}")))
}

pub fn mode_bits(raw: &str, what: &str) -> mlua::Result<u32> {
    let octal =
        (3..=4).contains(&raw.len()) && raw.bytes().all(|digit| (b'0'..b'8').contains(&digit));
    if !octal {
        return Err(external(format!(
            "{what} needs a `mode` of three or four octal digits, got `{raw}`"
        )));
    }

    Ok(raw
        .bytes()
        .fold(0, |bits, digit| bits * 8 + u32::from(digit - b'0')))
}

pub fn owner_name(raw: &str, what: &str) -> mlua::Result<String> {
    let mut parts = raw.split(':');
    let user = parts.next().unwrap_or_default();
    let group = parts.next();

    let broken = parts.next().is_some()
        || user.is_empty()
        || group.is_some_and(str::is_empty)
        || raw.contains(char::is_whitespace);
    if broken {
        return Err(external(format!(
            "{what} needs an `owner` like \"root\" or \"root:root\", got `{raw}`"
        )));
    }

    Ok(raw.to_string())
}

pub fn lookup<T: Copy>(entries: &[(&str, T)], name: &str, field: &str) -> mlua::Result<T> {
    entries
        .iter()
        .find(|(key, _)| *key == name)
        .map(|(_, value)| *value)
        .ok_or_else(|| {
            external(format!(
                "unknown {field} `{name}` (available: {})",
                keys(entries)
            ))
        })
}

pub fn link_mode(name: Option<String>) -> mlua::Result<Option<LinkMode>> {
    name.map(|name| lookup(&LINK_MODES, &name, "link mode"))
        .transpose()
}

pub fn conflict_policy(name: Option<String>) -> mlua::Result<Option<ConflictPolicy>> {
    name.map(|name| lookup(&CONFLICT_POLICIES, &name, "conflict policy"))
        .transpose()
}

fn keys<T>(entries: &[(&str, T)]) -> String {
    entries
        .iter()
        .map(|(key, _)| *key)
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;

    use super::super::constants::{CONFLICT_POLICIES, CRYPT_BACKENDS, LINK_MODES};

    const ENTRIES: [(&str, u8); 2] = [("one", 1), ("two", 2)];

    #[test]
    fn link_modes_round_trip_through_their_name() {
        for (name, mode) in LINK_MODES {
            assert_eq!(mode.name(), name);
        }
    }

    #[test]
    fn conflict_policies_round_trip_through_their_name() {
        for (name, policy) in CONFLICT_POLICIES {
            assert_eq!(policy.name(), name);
        }
    }

    #[test]
    fn crypt_backends_round_trip_through_their_name() {
        for (name, backend) in CRYPT_BACKENDS {
            assert_eq!(backend.name(), name);
        }
    }

    #[test]
    fn lookup_finds_a_known_name() {
        assert_eq!(lookup(&ENTRIES, "two", "number").unwrap(), 2);
    }

    #[test]
    fn lookup_lists_the_available_names() {
        let err = lookup(&ENTRIES, "three", "number").unwrap_err().to_string();

        assert!(err.contains("unknown number `three`"));
        assert!(err.contains("available: one, two"));
    }

    #[test]
    fn mode_bits_reads_three_or_four_octal_digits() {
        assert_eq!(mode_bits("600", "a rule").unwrap(), 0o600);
        assert_eq!(mode_bits("0644", "a rule").unwrap(), 0o644);
        assert_eq!(mode_bits("4755", "a rule").unwrap(), 0o4755);
    }

    #[test]
    fn mode_bits_rejects_anything_else() {
        for raw in ["60", "60000", "6o0", "800", "+60"] {
            let err = mode_bits(raw, "a rule").unwrap_err().to_string();

            assert!(err.contains("three or four octal digits"), "{raw}");
            assert!(err.contains(raw), "{raw}");
        }
    }

    #[test]
    fn owner_name_takes_a_user_with_an_optional_group() {
        assert_eq!(owner_name("root", "a rule").unwrap(), "root");
        assert_eq!(owner_name("root:wheel", "a rule").unwrap(), "root:wheel");
        assert_eq!(owner_name("0:0", "a rule").unwrap(), "0:0");
    }

    #[test]
    fn owner_name_rejects_a_broken_name() {
        for raw in ["", ":", "root:", ":wheel", "a:b:c", "ro ot"] {
            let err = owner_name(raw, "a rule").unwrap_err().to_string();

            assert!(err.contains("needs an `owner`"), "{raw}");
        }
    }

    #[test]
    fn regex_reports_an_invalid_expression() {
        assert!(
            regex("^[")
                .unwrap_err()
                .to_string()
                .contains("invalid regex `^[`")
        );
    }

    #[test]
    fn pattern_reports_an_invalid_glob() {
        assert!(
            pattern("[")
                .unwrap_err()
                .to_string()
                .contains("invalid pattern `[`")
        );
    }
}
