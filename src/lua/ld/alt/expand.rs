use std::path::Path;

use mlua::{Function, Lua, Table};

use super::super::constants::API;
use super::super::parse::external;
use super::super::surface::{self, Surface};
use super::constants::{EXPAND, NAMESPACE};
use super::file::resolve;
use crate::lua::embed;
use crate::lua::runtime::environment;

pub fn function(lua: &Lua) -> mlua::Result<Function> {
    lua.create_function(|lua, (name, vars): (String, Option<Table>)| {
        if surface::inert(lua, &format!("{NAMESPACE}.{EXPAND}"), Surface::Template) {
            return Ok(None);
        }

        let path = resolve(lua, &name, EXPAND)?;
        expand(lua, &path, vars).map(Some)
    })
}

fn expand(lua: &Lua, path: &Path, vars: Option<Table>) -> mlua::Result<String> {
    let source = std::fs::read_to_string(path).map_err(|err| {
        external(format!(
            "`{API}.{NAMESPACE}.{EXPAND}` failed to read {}: {err}",
            path.display()
        ))
    })?;

    let chunk = embed::compile(&source).map_err(|err| {
        external(format!(
            "`{API}.{NAMESPACE}.{EXPAND}` failed to compile {}: {err:#}",
            path.display()
        ))
    })?;

    embed::run(
        lua,
        chunk,
        &path.display().to_string(),
        environment(lua, vars)?,
    )
    .map_err(|err| {
        external(format!(
            "`{API}.{NAMESPACE}.{EXPAND}` failed to run {}: {err}",
            path.display()
        ))
    })
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
    fn the_vars_are_optional() {
        let root = tempfile::tempdir().unwrap();
        let dir = template(root.path());
        std::fs::write(dir.join("plain.tmpl.zsh"), "plain\n").unwrap();

        let outputs = from_template(&dir, r#"return ld.alt.expand("plain.tmpl.zsh")"#).unwrap();

        assert_eq!(outputs[0].content(), &Content::Text("plain\n".to_string()));
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
    fn an_absolute_path_is_reached() {
        let root = tempfile::tempdir().unwrap();
        let dir = template(root.path());
        let shared = root.path().join("shared.tmpl");
        std::fs::write(&shared, "shared <%= x %>").unwrap();

        let outputs = from_template(
            &dir,
            &format!(
                r#"return ld.alt.expand("{}", {{ x = 1 }})"#,
                shared.display()
            ),
        )
        .unwrap();

        assert_eq!(outputs[0].content(), &Content::Text("shared 1".to_string()));
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
