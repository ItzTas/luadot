use std::path::PathBuf;

use mlua::{Function, Lua, Table, Value};

use super::super::constants::{API, CONFLICT, LINK, MODE, ON_CHANGE};
use super::super::parse::{chain, conflict_policy, external, link_mode, mode_bits};
use super::super::surface::{self, Surface};
use super::constants::{CONTENT, DEST, DEST_ALONE, FILE, NAMESPACE, OUT};
use super::file::handle;
use crate::files::{Placement, sync_file, write_file};
use crate::hook::Hooks;
use crate::lua::{Content, Output, Scope};
use crate::utils::dry_run;

pub fn function(lua: &Lua) -> mlua::Result<Function> {
    lua.create_function(|lua, value: Value| {
        surface::slow_in(lua, &format!("{NAMESPACE}.{OUT}"), Surface::Config);

        output(lua, value)
    })
}

pub fn output(lua: &Lua, value: Value) -> mlua::Result<()> {
    let output = parse(lua, value)?;
    if Surface::current(lua) == Some(Surface::Template) {
        Scope::building(lua)?.add_output(output);
        return Ok(());
    }

    place(&output)
}

fn place(output: &Output) -> mlua::Result<()> {
    if dry_run() {
        return Ok(());
    }

    let call = format!("`{API}.{NAMESPACE}.{OUT}`");
    let policy = output.conflict().unwrap_or_default();

    let placement = Placement::new(output.link().unwrap_or_default()).with_mode(output.mode());
    let outcome = match output.content() {
        Content::Text(text) => write_file(policy, placement, output.dest(), text),
        Content::File(source) => sync_file(policy, placement, source, output.dest()),
    }
    .map_err(|err| {
        external(format!(
            "{call} failed to write {}: {err:#}",
            output.dest().display()
        ))
    })?;

    let mut hooks = Hooks::new(false);
    hooks.record(outcome, output.on_change());

    hooks.finish(&call).map_err(chain)
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
    let dest: Option<String> = entry.get(DEST)?;
    let link: Option<String> = entry.get(LINK)?;
    let conflict: Option<String> = entry.get(CONFLICT)?;
    let on_change: Option<String> = entry.get(ON_CHANGE)?;

    let content = content(&entry.get::<Value>(CONTENT)?)?;
    let mode = mode(&entry.get::<Value>(MODE)?, &content)?;

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
    Scope::building(lua)?
        .destination(raw)
        .ok_or_else(|| external(format!("`{API}.{NAMESPACE}.{OUT}` {DEST_ALONE}")))
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
    use std::path::Path;

    use super::super::super::{Paths, install};
    use super::super::fixture::{error, template};
    use super::*;
    use crate::lua::from_template;
    use crate::lua::runtime::runtime;
    use crate::state::Classes;

    fn script(dir: &Path, source: &str) -> mlua::Result<()> {
        let lua = runtime().unwrap();
        let paths = Paths::new(dir, dir).with_dir(dir);
        install(&lua, Surface::Bootstrap, &paths, &Classes::default()).unwrap();

        lua.load(source).exec()
    }

    #[test]
    fn a_script_writes_the_file_where_it_declares_it() {
        let root = tempfile::tempdir().unwrap();
        let dest = root.path().join("generated/motd");

        script(
            root.path(),
            &format!(
                r#"ld.alt.out({{ dest = "{}", content = "welcome\n", mode = "600" }})"#,
                dest.display()
            ),
        )
        .unwrap();

        assert_eq!(std::fs::read_to_string(&dest).unwrap(), "welcome\n");
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
