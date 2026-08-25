use mlua::{Function, Lua, Table, Value};

use super::super::constants::API;
use super::super::parse::external;
use super::command::Command;
use super::constants::{ARGS, DIFF_KEYS, NAMESPACE, TOOL};
use super::parse::{around, known, report};
use crate::lua::{Config, Diff, Tool};

pub fn function(lua: &Lua, command: Command) -> mlua::Result<Function> {
    lua.create_function(move |lua, options: Table| {
        let call = format!("{NAMESPACE}.{}", command.path());
        let diff = diff(&call, &options)?;
        let around = around(&call, &options)?;
        Config::building(lua, |config| {
            config.set_diff(diff);
            config.set_around(command, around);
        })
    })
}

fn diff(call: &str, options: &Table) -> mlua::Result<Diff> {
    known(call, options, &DIFF_KEYS)?;

    Ok(Diff::default()
        .with_report(report(call, options)?)
        .with_tool(tool(call, options)?)
        .with_args(words(call, options, ARGS)?))
}

fn tool(call: &str, options: &Table) -> mlua::Result<Option<Tool>> {
    let Some(words) = words(call, options, TOOL)? else {
        return Ok(None);
    };
    let Some((program, arguments)) = words.split_first() else {
        return Err(external(format!(
            "`{API}.{call}`: `{TOOL}` takes the program to run, got an empty list"
        )));
    };

    Ok(Some(Tool::new(program.clone(), arguments.to_vec())))
}

fn words(call: &str, options: &Table, key: &str) -> mlua::Result<Option<Vec<String>>> {
    let words = match options.get::<Value>(key)? {
        Value::Nil => return Ok(None),
        Value::String(word) => vec![word.to_str()?.to_string()],
        Value::Table(list) => list
            .sequence_values::<String>()
            .collect::<mlua::Result<Vec<String>>>()
            .map_err(|_| expected(call, key))?,
        _ => return Err(expected(call, key)),
    };

    if words.iter().any(|word| word.trim().is_empty()) {
        return Err(expected(call, key));
    }

    Ok(Some(words))
}

fn expected(call: &str, key: &str) -> mlua::Error {
    external(format!(
        "`{API}.{call}`: `{key}` takes a word or a list of words"
    ))
}

#[cfg(test)]
mod tests {
    use crate::lua::{Custom, from_source};

    #[test]
    fn the_function_survives_the_run() {
        let config = from_source(
            r#"ld.on.diff({ summary = function(counts) return counts.default .. "!" end })"#,
        )
        .unwrap();

        let counts = crate::lua::DiffCounts::new(
            crate::files::Side::Repository,
            1,
            2,
            "1 of 2 managed file(s) differ".to_string(),
        );
        let shown = config
            .diff()
            .summary()
            .unwrap()
            .shown("a hook", &counts)
            .unwrap();

        assert_eq!(shown, Some("1 of 2 managed file(s) differ!".to_string()));
    }

    #[test]
    fn false_silences_what_it_is_given_to() {
        let config =
            from_source("ld.on.diff({ summary = false, entry = false, render = false })").unwrap();

        assert!(matches!(config.diff().summary(), Some(Custom::Silent)));
        assert!(matches!(config.diff().entry(), Some(Custom::Silent)));
        assert!(matches!(config.diff().render(), Some(Custom::Silent)));
    }

    #[test]
    fn the_tool_carries_program_and_args() {
        let config =
            from_source(r#"ld.on.diff({ tool = { "delta", "--side-by-side" }, args = "--stat" })"#)
                .unwrap();
        let tool = config.diff().tool().unwrap();

        assert_eq!(tool.program(), "delta");
        assert_eq!(tool.arguments(), ["--side-by-side"]);
        assert_eq!(config.diff().args(), ["--stat"]);
    }
}
