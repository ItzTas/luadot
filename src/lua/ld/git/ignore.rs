use glob::Pattern;
use mlua::{Function, Lua, Table, Value};

use super::super::constants::API;
use super::super::parse::{external, pattern};
use super::super::surface::{self, Surface};
use super::constants::{IGNORE, NAMESPACE};
use crate::lua::Config;

pub fn function(lua: &Lua) -> mlua::Result<Function> {
    lua.create_function(|lua, value: Value| {
        if surface::inert(lua, &format!("{NAMESPACE}.{IGNORE}"), Surface::Config) {
            return Ok(());
        }

        let patterns = patterns(&value)?;
        Config::building(lua)?.add_ignore(patterns);
        Ok(())
    })
}

fn patterns(value: &Value) -> mlua::Result<Vec<Pattern>> {
    match value {
        Value::String(raw) => Ok(vec![pattern(&raw.to_str()?)?]),
        Value::Table(list) => from_list(list),
        other => Err(external(format!(
            "`{API}.{NAMESPACE}.ignore` takes a string or a table of strings, got {}",
            other.type_name()
        ))),
    }
}

fn from_list(list: &Table) -> mlua::Result<Vec<Pattern>> {
    list.clone()
        .sequence_values::<String>()
        .enumerate()
        .map(|(index, raw)| {
            let raw = raw.map_err(|_| {
                external(format!(
                    "`{API}.{NAMESPACE}.ignore` entry {} is not a string",
                    index + 1
                ))
            })?;
            pattern(&raw)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::lua::{Config, from_source};

    fn configure(source: &str) -> Config {
        from_source(source).unwrap()
    }

    fn error(source: &str) -> String {
        format!("{:#}", from_source(source).unwrap_err())
    }

    #[test]
    fn takes_a_list_or_a_single_pattern() {
        let config = configure(r#"ld.git.ignore({ "*.swp", ".cache/**" })"#);

        assert!(config.is_ignored(Path::new(".vimrc.swp")));
        assert!(config.is_ignored(Path::new(".cache/nvim/log")));
        assert!(!config.is_ignored(Path::new(".vimrc")));

        let config = configure(r#"ld.git.ignore(".netrc")"#);

        assert!(config.is_ignored(Path::new(".netrc")));
    }

    #[test]
    fn repeated_calls_accumulate() {
        let config = configure(
            r#"
            ld.git.ignore({ "*.swp" })
            ld.git.ignore("*.bak")
            "#,
        );

        assert!(config.is_ignored(Path::new(".vimrc.swp")));
        assert!(config.is_ignored(Path::new(".vimrc.bak")));
    }

    #[test]
    fn ignoring_a_directory_ignores_its_contents() {
        let config = configure(r#"ld.git.ignore({ ".config/nvim" })"#);

        assert!(config.is_ignored(Path::new(".config/nvim")));
        assert!(config.is_ignored(Path::new(".config/nvim/lua/plugins.lua")));
        assert!(!config.is_ignored(Path::new(".config/zsh/.zshrc")));
    }

    #[test]
    fn a_single_star_does_not_cross_a_separator() {
        let config = configure(r#"ld.git.ignore({ ".config/*" })"#);

        assert!(config.is_ignored(Path::new(".config/mimeapps.list")));
        assert!(config.is_ignored(Path::new(".config/nvim/init.lua")));
        assert!(!config.is_ignored(Path::new(".bashrc")));
    }

    #[test]
    fn rejects_an_invalid_pattern() {
        assert!(error(r#"ld.git.ignore({ "[" })"#).contains("invalid pattern `[`"));
    }

    #[test]
    fn rejects_an_argument_of_the_wrong_type() {
        assert!(error("ld.git.ignore(42)").contains("takes a string or a table of strings"));
    }
}
