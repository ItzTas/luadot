use std::path::Path;

use mlua::{Function, Lua, Table};

use super::constants::EXPAND;
use super::file::{failed, read, resolve};
use crate::lua::embed;
use crate::lua::runtime::environment;

pub fn function(lua: &Lua) -> mlua::Result<Function> {
    lua.create_function(|lua, (name, vars): (String, Option<Table>)| {
        let path = resolve(lua, &name, EXPAND)?;
        expand(lua, &path, vars)
    })
}

fn expand(lua: &Lua, path: &Path, vars: Option<Table>) -> mlua::Result<String> {
    let source = read(EXPAND, path)?;

    let chunk = embed::compile(&source)
        .map_err(|err| failed(EXPAND, "compile", path.display(), format!("{err:#}")))?;

    embed::run(
        lua,
        chunk,
        &path.display().to_string(),
        environment(lua, vars)?,
    )
    .map_err(|err| failed(EXPAND, "run", path.display(), err))
}

#[cfg(test)]
mod tests {
    use super::super::fixture::{error, template};
    use crate::lua::{Content, from_template};

    #[test]
    fn vars_and_interface_stay_in_scope() {
        let root = tempfile::tempdir().unwrap();
        let dir = template(root.path());
        std::fs::write(
            dir.join("zshrc.tmpl.zsh"),
            "export EDITOR=<%= editor %>\nhome is a <%= type(ld.path.home) %>\n",
        )
        .unwrap();

        let outputs = from_template(
            &dir,
            r#"return ld.alt.expand("zshrc.tmpl.zsh", { editor = "nvim" })"#,
        )
        .unwrap();

        assert_eq!(
            outputs[0].content(),
            &Content::Text("export EDITOR=nvim\nhome is a string\n".to_string())
        );
    }

    #[test]
    fn every_template_keeps_its_own_scope() {
        let root = tempfile::tempdir().unwrap();
        let dir = template(root.path());
        std::fs::write(dir.join("partial.tmpl"), "[<%= name %>]").unwrap();
        std::fs::write(
            dir.join("outer.tmpl"),
            "<%= name %><%= ld.alt.expand(\"partial.tmpl\", { name = \"inner\" }) %><%= name %>",
        )
        .unwrap();

        let outputs = from_template(
            &dir,
            r#"return ld.alt.expand("outer.tmpl", { name = "outer" })"#,
        )
        .unwrap();

        assert_eq!(
            outputs[0].content(),
            &Content::Text("outer[inner]outer".to_string())
        );
    }

    #[test]
    fn a_failing_partial_names_its_file() {
        let root = tempfile::tempdir().unwrap();
        let dir = template(root.path());
        std::fs::write(dir.join("partial.tmpl"), "fine\n<%= missing() %>\n").unwrap();
        std::fs::write(
            dir.join("outer.tmpl"),
            "<%= ld.alt.expand(\"partial.tmpl\") %>",
        )
        .unwrap();

        let err = error(&dir, r#"return ld.alt.expand("outer.tmpl")"#);

        assert!(err.contains("partial.tmpl"));
        assert!(err.contains(":2:"));
    }
}
