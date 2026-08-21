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
    fn the_vars_and_the_interface_stay_in_scope() {
        let root = tempfile::tempdir().unwrap();
        let dir = template(root.path());
        std::fs::write(
            dir.join("zshrc.tmpl.zsh"),
            "export EDITOR=<%= editor %>\nhost is a <%= type(ld.sys.host.name) %>\n",
        )
        .unwrap();

        let outputs = from_template(
            &dir,
            r#"return ld.alt.expand("zshrc.tmpl.zsh", { editor = "nvim" })"#,
        )
        .unwrap();

        assert_eq!(
            outputs[0].content(),
            &Content::Text("export EDITOR=nvim\nhost is a string\n".to_string())
        );
    }

    #[test]
    fn a_missing_file_is_reported() {
        let root = tempfile::tempdir().unwrap();
        let dir = template(root.path());

        let err = error(&dir, r#"return ld.alt.expand("missing.tmpl")"#);

        assert!(err.contains("`ld.alt.expand`"));
        assert!(err.contains("found no file `missing.tmpl`"));
    }

    #[test]
    fn a_template_expands_another_one() {
        let root = tempfile::tempdir().unwrap();
        let dir = template(root.path());
        std::fs::write(dir.join("header.tmpl.zsh"), "# <%= title %>\n").unwrap();
        std::fs::write(
            dir.join("zshrc.tmpl.zsh"),
            "<%= ld.alt.expand(\"header.tmpl.zsh\", { title = \"zsh\" }) -%>\nexport EDITOR=<%= editor %>\n",
        )
        .unwrap();

        let outputs = from_template(
            &dir,
            r#"return ld.alt.expand("zshrc.tmpl.zsh", { editor = "nvim" })"#,
        )
        .unwrap();

        assert_eq!(
            outputs[0].content(),
            &Content::Text("# zsh\nexport EDITOR=nvim\n".to_string())
        );
    }

    #[test]
    fn every_template_keeps_its_own_vars_and_its_own_buffer() {
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
    fn a_partial_that_fails_reports_its_own_file() {
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

    #[test]
    fn a_broken_template_reports_its_own_line() {
        let root = tempfile::tempdir().unwrap();
        let dir = template(root.path());
        std::fs::write(dir.join("broken.tmpl"), "fine\n<%= missing() %>\n").unwrap();

        let err = error(&dir, r#"return ld.alt.expand("broken.tmpl")"#);

        assert!(err.contains("`ld.alt.expand` failed to run"));
        assert!(err.contains(":2:"));
    }

    #[test]
    fn an_unterminated_tag_is_a_compile_error() {
        let root = tempfile::tempdir().unwrap();
        let dir = template(root.path());
        std::fs::write(dir.join("broken.tmpl"), "text\n<% x = 1").unwrap();

        let err = error(&dir, r#"return ld.alt.expand("broken.tmpl")"#);

        assert!(err.contains("`ld.alt.expand` failed to compile"));
        assert!(err.contains("line 2"));
    }
}
