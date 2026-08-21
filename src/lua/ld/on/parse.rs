use mlua::{Table, Value};

use super::super::constants::API;
use super::super::parse::external;
use super::constants::{ENTRY, RENDER, SUMMARY};
use crate::lua::config::constants::{AFTER, BEFORE};
use crate::lua::{Around, Call, Custom, Report};

pub fn report(call: &str, options: &Table) -> mlua::Result<Report> {
    Ok(Report::default()
        .with_entry(custom(call, options, ENTRY, false)?)
        .with_summary(custom(call, options, SUMMARY, true)?)
        .with_render(custom(call, options, RENDER, false)?))
}

pub fn around(call: &str, options: &Table) -> mlua::Result<Around> {
    Ok(Around::default()
        .with_before(custom(call, options, BEFORE, false)?)
        .with_after(custom(call, options, AFTER, false)?))
}

pub fn custom(call: &str, options: &Table, key: &str, texts: bool) -> mlua::Result<Option<Custom>> {
    match options.get::<Value>(key)? {
        Value::Nil => Ok(None),
        Value::Boolean(false) => Ok(Some(Custom::Silent)),
        Value::Function(function) => Ok(Some(Custom::Call(Call::new(function)))),
        Value::String(text) if texts => Ok(Some(Custom::Text(text.to_str()?.to_string()))),
        other => Err(external(format!(
            "`{API}.{call}`: `{key}` takes {}, got {}",
            kinds(texts),
            other.type_name()
        ))),
    }
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

fn kinds(texts: bool) -> &'static str {
    match texts {
        true => "a function, a string or false",
        false => "a function or false",
    }
}
