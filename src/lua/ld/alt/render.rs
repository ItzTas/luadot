use std::path::Path;

use mlua::{Function, Lua, Table, Value};

use super::super::constants::API;
use super::super::parse::external;
use super::constants::{NAMESPACE, RENDER};
use super::file::{failed, read, resolve};
use crate::lua::runtime::environment;

pub fn function(lua: &Lua) -> mlua::Result<Function> {
    lua.create_function(|lua, (name, vars): (String, Option<Table>)| {
        let path = resolve(lua, &name, RENDER)?;
        render(lua, &path, vars)
    })
}

fn render(lua: &Lua, path: &Path, vars: Option<Table>) -> mlua::Result<String> {
    let source = read(RENDER, path)?;

    let rendered: Value = lua
        .load(source)
        .set_name(path.display().to_string())
        .set_environment(environment(lua, vars)?)
        .eval()
        .map_err(|err| failed(RENDER, "run", path.display(), err))?;

    match rendered {
        Value::String(text) => Ok(text.to_str()?.to_string()),
        other => Err(external(format!(
            "`{API}.{NAMESPACE}.{RENDER}` expects {} to return a string, got {}",
            path.display(),
            other.type_name()
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::super::fixture::template;
    use crate::lua::{Content, from_template};

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
}
