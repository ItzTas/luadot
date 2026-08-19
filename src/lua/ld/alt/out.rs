use std::path::PathBuf;

use mlua::{Function, Lua, Table, Value};

use super::super::constants::API;
use super::super::parse::{conflict_policy, external, link_mode, mode_bits};
use super::super::surface::{self, Surface};
use super::constants::{FILE, NAMESPACE, OUT};
use super::file::handle;
use crate::lua::{Content, Output, Template};

pub fn function(lua: &Lua) -> mlua::Result<Function> {
    lua.create_function(|lua, value: Value| {
        if surface::inert(lua, &format!("{NAMESPACE}.{OUT}"), Surface::Template) {
            return Ok(());
        }

        output(lua, value)
    })
}

pub fn output(lua: &Lua, value: Value) -> mlua::Result<()> {
    let output = parse(lua, value)?;
    Template::building(lua)?.add_output(output);

    Ok(())
}

fn parse(lua: &Lua, value: Value) -> mlua::Result<Output> {
    let Value::Table(entry) = value else {
        return Ok(Output::new(
            destination(lua, None)?,
            content(&value)?,
            None,
            None,
        ));
    };

    from_table(lua, &entry)
}

fn from_table(lua: &Lua, entry: &Table) -> mlua::Result<Output> {
    let dest: Option<String> = entry.get("dest")?;
    let link: Option<String> = entry.get("link")?;
    let conflict: Option<String> = entry.get("conflict")?;
    let on_change: Option<String> = entry.get("on_change")?;

    let content = content(&entry.get::<Value>("content")?)?;
    let mode = mode(&entry.get::<Value>("mode")?, &content)?;

    Ok(Output::new(
        destination(lua, dest.as_deref())?,
        content,
        link_mode(link)?,
        conflict_policy(conflict)?,
    )
    .with_mode(mode)
    .with_on_change(on_change))
}

fn mode(value: &Value, content: &Content) -> mlua::Result<Option<u32>> {
    let raw = match value {
        Value::Nil => return Ok(None),
        Value::String(text) => text.to_str()?.to_string(),
        other => {
            return Err(external(format!(
                "`{API}.{NAMESPACE}.{OUT}` needs a `mode` holding an octal string like \"600\", got {}",
                other.type_name()
            )));
        }
    };

    if let Content::File(path) = content {
        return Err(external(format!(
            "`{API}.{NAMESPACE}.{OUT}` cannot set a `mode` on `{API}.{NAMESPACE}.{FILE}`: {} is the repository's own copy",
            path.display()
        )));
    }

    mode_bits(&raw, &format!("`{API}.{NAMESPACE}.{OUT}`")).map(Some)
}

fn destination(lua: &Lua, raw: Option<&str>) -> mlua::Result<PathBuf> {
    Ok(Template::building(lua)?.destination(raw))
}

fn content(value: &Value) -> mlua::Result<Content> {
    match value {
        Value::String(text) => Ok(Content::Text(text.to_str()?.to_string())),
        Value::UserData(data) => Ok(Content::File(handle(data)?)),
        other => Err(external(format!(
            "`{API}.{NAMESPACE}.{OUT}` needs a `content` holding a string or `{API}.{NAMESPACE}.{FILE}`, got {}",
            other.type_name()
        ))),
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use crate::lua::from_template;

    fn error(dir: &Path, source: &str) -> String {
        format!("{:#}", from_template(dir, source).unwrap_err())
    }

    fn template(root: &Path) -> PathBuf {
        let dir = root.join(".zshrc.luadot");
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn rejects_a_table_without_content() {
        let root = tempfile::tempdir().unwrap();
        let dir = template(root.path());

        let err = error(&dir, r#"ld.alt.out({ dest = "~/.zshrc" })"#);

        assert!(err.contains("needs a `content`"));
        assert!(err.contains("got nil"));
    }

    #[test]
    fn rejects_an_unknown_link_mode() {
        let root = tempfile::tempdir().unwrap();
        let dir = template(root.path());

        let err = error(&dir, r#"ld.alt.out({ content = "x", link = "magic" })"#);

        assert!(err.contains("unknown link mode `magic`"));
    }

    #[test]
    fn rejects_an_unknown_conflict_policy() {
        let root = tempfile::tempdir().unwrap();
        let dir = template(root.path());

        let err = error(&dir, r#"ld.alt.out({ content = "x", conflict = "ask" })"#);

        assert!(err.contains("unknown conflict policy `ask`"));
    }

    #[test]
    fn a_mode_is_read_as_octal() {
        let root = tempfile::tempdir().unwrap();
        let dir = template(root.path());

        let outputs =
            from_template(&dir, r#"ld.alt.out({ content = "x", mode = "600" })"#).unwrap();

        assert_eq!(outputs[0].mode(), Some(0o600));

        let outputs =
            from_template(&dir, r#"ld.alt.out({ content = "x", mode = "0644" })"#).unwrap();

        assert_eq!(outputs[0].mode(), Some(0o644));
    }

    #[test]
    fn a_file_carries_no_mode_of_its_own() {
        let root = tempfile::tempdir().unwrap();
        let dir = template(root.path());

        let outputs = from_template(&dir, r#"ld.alt.out({ content = "x" })"#).unwrap();

        assert_eq!(outputs[0].mode(), None);
    }

    #[test]
    fn rejects_a_mode_that_is_not_three_or_four_octal_digits() {
        let root = tempfile::tempdir().unwrap();
        let dir = template(root.path());

        for raw in ["60", "60000", "6o0", "800", "+60"] {
            let err = error(
                &dir,
                &format!(r#"ld.alt.out({{ content = "x", mode = "{raw}" }})"#),
            );

            assert!(err.contains("three or four octal digits"), "{raw}");
            assert!(err.contains(raw), "{raw}");
        }
    }

    #[test]
    fn rejects_a_mode_that_is_not_a_string() {
        let root = tempfile::tempdir().unwrap();
        let dir = template(root.path());

        let err = error(&dir, r#"ld.alt.out({ content = "x", mode = 600 })"#);

        assert!(err.contains("needs a `mode` holding an octal string like \"600\""));
    }

    #[test]
    fn rejects_a_mode_on_a_file_of_the_repository() {
        let root = tempfile::tempdir().unwrap();
        let dir = template(root.path());
        std::fs::write(dir.join("netrc"), "machine example").unwrap();

        let err = error(
            &dir,
            r#"ld.alt.out({ content = ld.alt.file("netrc"), mode = "600" })"#,
        );

        assert!(err.contains("cannot set a `mode` on `ld.alt.file`"));
        assert!(err.contains("netrc"));
    }

    #[test]
    fn a_command_runs_when_the_file_changes() {
        let root = tempfile::tempdir().unwrap();
        let dir = template(root.path());

        let outputs = from_template(
            &dir,
            r#"ld.alt.out({ content = "x", on_change = "systemctl --user restart mako" })"#,
        )
        .unwrap();

        assert_eq!(
            outputs[0].on_change(),
            Some("systemctl --user restart mako")
        );
    }
}
