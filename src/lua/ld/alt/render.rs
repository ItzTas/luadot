use std::path::Path;

use mlua::{Function, Lua, Table, Value};

use super::super::constants::API;
use super::super::parse::external;
use super::super::surface::{self, Surface};
use super::constants::{NAMESPACE, RENDER};
use super::file::resolve;

pub fn function(lua: &Lua) -> mlua::Result<Function> {
    lua.create_function(|lua, (name, vars): (String, Option<Table>)| {
        if surface::inert(lua, &format!("{NAMESPACE}.{RENDER}"), Surface::Template) {
            return Ok(None);
        }

        let path = resolve(lua, &name, RENDER)?;
        render(lua, &path, vars).map(Some)
    })
}

fn render(lua: &Lua, path: &Path, vars: Option<Table>) -> mlua::Result<String> {
    let source = std::fs::read_to_string(path).map_err(|err| {
        external(format!(
            "`{API}.{NAMESPACE}.{RENDER}` failed to read {}: {err}",
            path.display()
        ))
    })?;

    let rendered: Value = lua
        .load(source)
        .set_name(path.display().to_string())
        .set_environment(environment(lua, vars)?)
        .eval()
        .map_err(|err| {
            external(format!(
                "`{API}.{NAMESPACE}.{RENDER}` failed to run {}: {err}",
                path.display()
            ))
        })?;

    match rendered {
        Value::String(text) => Ok(text.to_str()?.to_string()),
        other => Err(external(format!(
            "`{API}.{NAMESPACE}.{RENDER}` expects {} to return a string, got {}",
            path.display(),
            other.type_name()
        ))),
    }
}

fn environment(lua: &Lua, vars: Option<Table>) -> mlua::Result<Table> {
    let environment = lua.create_table()?;
    if let Some(vars) = vars {
        for pair in vars.pairs::<Value, Value>() {
            let (name, value) = pair?;
            environment.set(name, value)?;
        }
    }

    let meta = lua.create_table()?;
    meta.set("__index", lua.globals())?;
    environment.set_metatable(Some(meta))?;

    Ok(environment)
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use crate::lua::{Content, from_template};

    fn template(root: &Path) -> PathBuf {
        let dir = root.join(".zshrc.luadot");
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn error(dir: &Path, source: &str) -> String {
        format!("{:#}", from_template(dir, source).unwrap_err())
    }

    #[test]
    fn the_standard_library_stays_reachable_from_a_rendered_file() {
        let root = tempfile::tempdir().unwrap();
        let dir = template(root.path());
        std::fs::write(
            dir.join("zshrc.tmpl.zsh"),
            r#"return table.concat({ "export EDITOR=", editor }, "")"#,
        )
        .unwrap();

        let outputs = from_template(
            &dir,
            r#"return ld.alt.render("zshrc.tmpl.zsh", { editor = "nvim" })"#,
        )
        .unwrap();

        assert_eq!(
            outputs[0].content(),
            &Content::Text("export EDITOR=nvim".to_string())
        );
    }

    #[test]
    fn variables_are_optional() {
        let root = tempfile::tempdir().unwrap();
        let dir = template(root.path());
        std::fs::write(dir.join("plain.tmpl.zsh"), r#"return "plain""#).unwrap();

        let outputs = from_template(&dir, r#"return ld.alt.render("plain.tmpl.zsh")"#).unwrap();

        assert_eq!(outputs[0].content(), &Content::Text("plain".to_string()));
    }

    #[test]
    fn rejects_a_file_that_returns_no_string() {
        let root = tempfile::tempdir().unwrap();
        let dir = template(root.path());
        std::fs::write(dir.join("broken.tmpl.zsh"), "return 42").unwrap();

        let err = error(&dir, r#"return ld.alt.render("broken.tmpl.zsh")"#);

        assert!(err.contains("expects"));
        assert!(err.contains("to return a string, got integer"));
    }

    #[test]
    fn reports_a_file_that_fails_to_run() {
        let root = tempfile::tempdir().unwrap();
        let dir = template(root.path());
        std::fs::write(dir.join("broken.tmpl.zsh"), "error('boom')").unwrap();

        let err = error(&dir, r#"return ld.alt.render("broken.tmpl.zsh")"#);

        assert!(err.contains("failed to run"));
        assert!(err.contains("boom"));
    }
}
