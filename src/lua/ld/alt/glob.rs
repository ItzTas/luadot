use std::path::{Path, PathBuf};

use glob::Pattern;
use mlua::{Function, Lua};

use super::super::constants::API;
use super::super::parse::external;
use super::super::surface::{self, Surface};
use super::constants::{GLOB, NAMESPACE};
use crate::lua::Template;

pub fn function(lua: &Lua) -> mlua::Result<Function> {
    lua.create_function(|lua, pattern: String| {
        if surface::inert(lua, &format!("{NAMESPACE}.{GLOB}"), Surface::Template) {
            return Ok(None);
        }

        let dir = Template::building(lua)?.dir().to_path_buf();
        let names = names(&dir, &pattern)?;

        lua.create_sequence_from(names).map(Some)
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
    use super::*;
    use crate::lua::{Content, from_template};

    fn template(root: &Path) -> PathBuf {
        let dir = root.join(".zshrc.luadot");
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

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
    fn lists_the_matching_files_in_order() {
        let root = tempfile::tempdir().unwrap();
        let dir = template(root.path());
        write(&dir.join("conf.d/20-path.zsh"), "path");
        write(&dir.join("conf.d/10-env.zsh"), "env");
        write(&dir.join("conf.d/notes.md"), "notes");

        assert_eq!(
            listed(&dir, "conf.d/*.zsh"),
            "conf.d/10-env.zsh,conf.d/20-path.zsh"
        );
    }

    #[test]
    fn a_directory_is_never_listed() {
        let root = tempfile::tempdir().unwrap();
        let dir = template(root.path());
        std::fs::create_dir(dir.join("conf.d")).unwrap();
        write(&dir.join("conf.d/10-env.zsh"), "env");

        assert_eq!(listed(&dir, "*"), "");
        assert_eq!(listed(&dir, "conf.d/*"), "conf.d/10-env.zsh");
    }

    #[test]
    fn a_star_stays_inside_one_segment_and_two_cross_them() {
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
    fn the_files_it_names_are_the_ones_read_reads() {
        let root = tempfile::tempdir().unwrap();
        let dir = template(root.path());
        write(&dir.join("conf.d/10-env.zsh"), "export A=1\n");
        write(&dir.join("conf.d/20-path.zsh"), "path+=(~/bin)\n");

        let outputs = from_template(
            &dir,
            r#"
            local parts = {}
            for _, name in ipairs(ld.alt.glob("conf.d/*.zsh")) do
              parts[#parts + 1] = ld.alt.read(name)
            end
            return table.concat(parts, "")
            "#,
        )
        .unwrap();

        assert_eq!(
            outputs[0].content(),
            &Content::Text("export A=1\npath+=(~/bin)\n".to_string())
        );
    }

    #[test]
    fn a_pattern_climbing_out_names_what_read_reaches() {
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

    #[test]
    fn an_absolute_pattern_yields_absolute_names() {
        let root = tempfile::tempdir().unwrap();
        let dir = template(root.path());
        let shared = root.path().join("shared");
        write(&shared.join("aliases.zsh"), "alias ll='ls -l'\n");

        assert_eq!(
            listed(&dir, &format!("{}/*.zsh", shared.display())),
            shared.join("aliases.zsh").display().to_string()
        );
    }

    #[test]
    fn an_invalid_pattern_is_reported() {
        let root = tempfile::tempdir().unwrap();
        let dir = template(root.path());

        let err = format!(
            "{:#}",
            from_template(&dir, r#"return ld.alt.glob("conf.d/[")"#).unwrap_err()
        );

        assert!(err.contains("`ld.alt.glob` got an invalid pattern `conf.d/[`"));
    }
}
