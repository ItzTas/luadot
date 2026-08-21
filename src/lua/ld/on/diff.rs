use mlua::{Function, Lua, Table, Value};

use super::super::constants::API;
use super::super::parse::external;
use super::constants::{ARGS, DIFF, DIFF_KEYS, NAMESPACE, TOOL};
use super::parse::{known, report};
use crate::lua::{Config, Diff, Tool};

pub fn function(lua: &Lua) -> mlua::Result<Function> {
    lua.create_function(|lua, options: Table| {
        let call = format!("{NAMESPACE}.{DIFF}");
        let diff = diff(&call, &options)?;
        Config::building(lua, |config| config.set_diff(diff))?;

        Ok(())
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
    fn nothing_is_customized_until_the_call_runs() {
        let config = from_source("local unused = 1").unwrap();
        let diff = config.diff();

        assert!(diff.entry().is_none());
        assert!(diff.summary().is_none());
        assert!(diff.render().is_none());
        assert!(diff.tool().is_none());
        assert!(diff.args().is_empty());
    }

    #[test]
    fn a_function_is_kept_callable_after_the_configuration_ran() {
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
    fn the_tool_is_the_program_and_the_arguments_it_carries() {
        let config =
            from_source(r#"ld.on.diff({ tool = { "delta", "--side-by-side" }, args = "--stat" })"#)
                .unwrap();
        let tool = config.diff().tool().unwrap();

        assert_eq!(tool.program(), "delta");
        assert_eq!(tool.arguments(), ["--side-by-side"]);
        assert_eq!(config.diff().args(), ["--stat"]);
    }

    #[test]
    fn rejects_an_unknown_key() {
        let err = format!(
            "{:#}",
            from_source("ld.on.diff({ entries = false })").unwrap_err()
        );

        assert!(err.contains("`ld.on.diff`: unknown key `entries`"));
        assert!(err.contains("available: args, entry, render, summary, tool"));
    }

    #[test]
    fn rejects_a_value_the_key_does_not_accept() {
        let err = format!(
            "{:#}",
            from_source(r#"ld.on.diff({ entry = "a line" })"#).unwrap_err()
        );

        assert!(err.contains("`entry` takes a function or false"));

        let err = format!(
            "{:#}",
            from_source("ld.on.diff({ summary = 1 })").unwrap_err()
        );

        assert!(err.contains("`summary` takes a function, a string or false"));
    }

    #[test]
    fn rejects_a_tool_that_names_no_program() {
        let err = format!(
            "{:#}",
            from_source("ld.on.diff({ tool = {} })").unwrap_err()
        );

        assert!(err.contains("`tool` takes the program to run"));

        let err = format!(
            "{:#}",
            from_source(r#"ld.on.diff({ tool = "  " })"#).unwrap_err()
        );

        assert!(err.contains("`tool` takes a word or a list of words"));
    }
}
