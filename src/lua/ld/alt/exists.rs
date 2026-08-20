use mlua::{Function, Lua};

use super::super::surface::{self, Surface};
use super::constants::{EXISTS, NAMESPACE};
use crate::lua::Template;

pub fn function(lua: &Lua) -> mlua::Result<Function> {
    lua.create_function(|lua, name: String| {
        if surface::inert(lua, &format!("{NAMESPACE}.{EXISTS}"), Surface::Template) {
            return Ok(false);
        }

        Ok(Template::building(lua)?.resolve(&name).is_some())
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

    fn answer(dir: &Path, name: &str) -> String {
        let outputs =
            from_template(dir, &format!(r#"return tostring(ld.alt.exists("{name}"))"#)).unwrap();

        match outputs[0].content() {
            Content::Text(text) => text.clone(),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn a_directory_is_not_a_file() {
        let root = tempfile::tempdir().unwrap();
        let dir = template(root.path());
        std::fs::create_dir(dir.join("variants")).unwrap();

        assert_eq!(answer(&dir, "variants"), "false");
    }

    #[test]
    fn a_fallback_reaches_the_file_that_is_there() {
        let root = tempfile::tempdir().unwrap();
        let dir = template(root.path());
        std::fs::write(dir.join("default.zsh"), "default").unwrap();

        let outputs = from_template(
            &dir,
            r#"
            local name = ld.alt.exists("laptop.zsh") and "laptop.zsh" or "default.zsh"
            return ld.alt.file(name)
            "#,
        )
        .unwrap();

        assert_eq!(
            outputs[0].content(),
            &Content::File(dir.join("default.zsh"))
        );
    }
}
