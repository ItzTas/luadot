use std::path::PathBuf;

use mlua::{Function, Lua, Table, Value};

use super::super::constants::{API, CONFLICT_POLICIES, LINK_MODES};
use super::super::parse::{external, lookup};
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

    Ok(Output::new(
        destination(lua, dest.as_deref())?,
        content(&entry.get::<Value>("content")?)?,
        link.map(|name| lookup(&LINK_MODES, &name, "link mode"))
            .transpose()?,
        conflict
            .map(|name| lookup(&CONFLICT_POLICIES, &name, "conflict policy"))
            .transpose()?,
    ))
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
}
