use mlua::{Function, Lua};

use super::constants::READ;
use super::file::{read, resolve};

pub fn function(lua: &Lua) -> mlua::Result<Function> {
    lua.create_function(|lua, name: String| {
        let path = resolve(lua, &name, READ)?;
        read(READ, &path)
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
    fn yields_the_bytes_of_a_file_of_the_template() {
        let root = tempfile::tempdir().unwrap();
        let dir = template(root.path());
        std::fs::write(dir.join("aliases.zsh"), "alias ll='ls -l'\n").unwrap();

        let outputs = from_template(&dir, r#"return ld.alt.read("aliases.zsh")"#).unwrap();

        assert_eq!(
            outputs[0].content(),
            &Content::Text("alias ll='ls -l'\n".to_string())
        );
    }

    #[test]
    fn what_it_reads_is_never_run() {
        let root = tempfile::tempdir().unwrap();
        let dir = template(root.path());
        std::fs::write(dir.join("fragment.zsh"), "error('boom')\n").unwrap();

        let outputs = from_template(&dir, r#"return ld.alt.read("fragment.zsh")"#).unwrap();

        assert_eq!(
            outputs[0].content(),
            &Content::Text("error('boom')\n".to_string())
        );
    }

    #[test]
    fn a_missing_file_is_reported() {
        let root = tempfile::tempdir().unwrap();
        let dir = template(root.path());

        let err = error(&dir, r#"return ld.alt.read("missing.zsh")"#);

        assert!(err.contains("`ld.alt.read`"));
        assert!(err.contains("found no file `missing.zsh`"));
    }

    #[test]
    fn an_absolute_path_is_reached() {
        let root = tempfile::tempdir().unwrap();
        let dir = template(root.path());
        let shared = root.path().join("shared.zsh");
        std::fs::write(&shared, "shared\n").unwrap();

        let outputs = from_template(
            &dir,
            &format!(r#"return ld.alt.read("{}")"#, shared.display()),
        )
        .unwrap();

        assert_eq!(outputs[0].content(), &Content::Text("shared\n".to_string()));
    }
}
