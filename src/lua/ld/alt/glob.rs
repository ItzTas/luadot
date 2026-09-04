use std::path::{Path, PathBuf};

use glob::Pattern;
use mlua::{Function, Lua};

use super::super::constants::API;
use super::super::parse::external;
use super::constants::{GLOB, NAMESPACE};
use crate::lua::Scope;

pub fn function(lua: &Lua) -> mlua::Result<Function> {
    lua.create_function(|lua, pattern: String| {
        let dir = Scope::building(lua)?.dir().to_path_buf();
        let names = names(&dir, &pattern)?;

        lua.create_sequence_from(names)
    })
}

fn names(dir: &Path, pattern: &str) -> mlua::Result<Vec<String>> {
    let mut names: Vec<String> = walk(dir, pattern)?
        .iter()
        .filter(|path| path.is_file())
        .map(|path| name(dir, path))
        .collect();
    names.sort();

    Ok(names)
}

fn walk(dir: &Path, pattern: &str) -> mlua::Result<Vec<PathBuf>> {
    glob::glob(&rooted(dir, pattern))
        .map_err(|err| {
            external(format!(
                "`{API}.{NAMESPACE}.{GLOB}` got an invalid pattern `{pattern}`: {err}"
            ))
        })?
        .collect::<Result<Vec<PathBuf>, _>>()
        .map_err(|err| {
            external(format!(
                "`{API}.{NAMESPACE}.{GLOB}` failed to walk `{pattern}`: {err}"
            ))
        })
}

fn rooted(dir: &Path, pattern: &str) -> String {
    if Path::new(pattern).is_absolute() {
        return pattern.to_string();
    }

    format!("{}/{pattern}", Pattern::escape(&dir.display().to_string()))
}

fn name(dir: &Path, path: &Path) -> String {
    path.strip_prefix(dir).unwrap_or(path).display().to_string()
}

#[cfg(test)]
mod tests {
    use super::super::fixture::template;
    use super::*;
    use crate::lua::{Content, from_template};

    fn write(path: &Path, contents: &str) {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(path, contents).unwrap();
    }

    fn listed(dir: &Path, pattern: &str) -> String {
        let outputs = from_template(
            dir,
            &format!(r#"return table.concat(ld.alt.glob("{pattern}"), ",")"#),
        )
        .unwrap();

        match outputs[0].content() {
            Content::Text(text) => text.clone(),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn one_star_stays_in_a_segment() {
        let root = tempfile::tempdir().unwrap();
        let dir = template(root.path());
        write(&dir.join("conf.d/nested/deep.zsh"), "deep");
        write(&dir.join("conf.d/10-env.zsh"), "env");

        assert_eq!(listed(&dir, "conf.d/*.zsh"), "conf.d/10-env.zsh");
        assert_eq!(
            listed(&dir, "conf.d/**/*.zsh"),
            "conf.d/10-env.zsh,conf.d/nested/deep.zsh"
        );
    }

    #[test]
    fn a_pattern_climbing_out_is_readable() {
        let root = tempfile::tempdir().unwrap();
        let dir = template(root.path());
        write(
            &root.path().join("shared/aliases.zsh"),
            "alias ll='ls -l'\n",
        );

        assert_eq!(listed(&dir, "../shared/*.zsh"), "../shared/aliases.zsh");

        let outputs = from_template(
            &dir,
            r#"return ld.alt.read(ld.alt.glob("../shared/*.zsh")[1])"#,
        )
        .unwrap();

        assert_eq!(
            outputs[0].content(),
            &Content::Text("alias ll='ls -l'\n".to_string())
        );
    }
}
