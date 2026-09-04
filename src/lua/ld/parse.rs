use glob::Pattern;
use mlua::{Table, Value};
use regex::Regex;

use super::constants::{
    API, CONFLICT_POLICIES, LINK_MODES, MATCH, REGEX, SPECIAL_BITS, TRACK_KINDS,
};
use crate::files::{ConflictPolicy, LinkMode};
use crate::lua::{Matcher, Track};

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

pub fn special_bits(bits: u32) -> Option<String> {
    let names: Vec<&str> = SPECIAL_BITS
        .iter()
        .filter(|(bit, _)| bits & bit != 0)
        .map(|(_, name)| *name)
        .collect();

    (!names.is_empty()).then(|| names.join(" and "))
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
            "{what} needs an `owner` like \"user\" or \"user:group\", got `{raw}`"
        )));
    }

    Ok(raw.to_string())
}

pub fn known(call: &str, options: &Table, keys: &[&str]) -> mlua::Result<()> {
    for pair in options.clone().pairs::<String, Value>() {
        let (key, _) =
            pair.map_err(|_| external(format!("`{API}.{call}` takes a table of options")))?;

        if !keys.contains(&key.as_str()) {
            return Err(external(format!(
                "`{API}.{call}`: unknown key `{key}` (available: {})",
                keys.join(", ")
            )));
        }
    }

    Ok(())
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

pub fn track(name: Option<String>) -> mlua::Result<Option<Track>> {
    name.map(|name| lookup(&TRACK_KINDS, &name, "track kind"))
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

    #[test]
    fn mode_bits_reads_octal_digits() {
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
    fn special_bits_name_what_the_mode_asks_for() {
        assert_eq!(special_bits(0o644), None);
        assert_eq!(special_bits(0o1777), None);
        assert_eq!(special_bits(0o4755).as_deref(), Some("setuid"));
        assert_eq!(special_bits(0o2755).as_deref(), Some("setgid"));
        assert_eq!(special_bits(0o6755).as_deref(), Some("setuid and setgid"));
    }

    #[test]
    fn owner_name_rejects_a_broken_name() {
        for raw in ["", ":", "me:", ":wheel", "a:b:c", "m e"] {
            let err = owner_name(raw, "a rule").unwrap_err().to_string();

            assert!(err.contains("needs an `owner`"), "{raw}");
        }
    }
}
