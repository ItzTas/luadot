use std::path::Path;

use mlua::{Function, Lua, Value};

use super::constants::WRITE;
use super::resolve::{failed, resolve};

pub fn function(lua: &Lua) -> mlua::Result<Function> {
    lua.create_function(|lua, (value, text): (Value, mlua::String)| {
        let path = resolve(lua, &value, WRITE)?;

        write(&path, &text.as_bytes())
    })
}

fn write(path: &Path, bytes: &[u8]) -> mlua::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|err| failed(WRITE, "create", parent.display(), err))?;
    }

    std::fs::write(path, bytes).map_err(|err| failed(WRITE, "write", path.display(), err))
}

#[cfg(test)]
mod tests {
    use super::super::fixture::eval;

    #[test]
    fn writes_the_text_and_the_directories_leading_to_the_file() {
        let home = tempfile::tempdir().unwrap();

        eval(
            home.path(),
            r#"fs.write("plugins/lock.json", "{ \"v\": 1 }\n") return "done""#,
        )
        .unwrap();

        assert_eq!(
            std::fs::read_to_string(home.path().join("plugins/lock.json")).unwrap(),
            "{ \"v\": 1 }\n"
        );
    }

    #[test]
    fn a_path_that_cannot_be_written_is_reported() {
        let home = tempfile::tempdir().unwrap();
        std::fs::write(home.path().join("plugins"), "a file, not a directory").unwrap();

        let err = eval(home.path(), r#"fs.write("plugins/lock.json", "")"#)
            .unwrap_err()
            .to_string();

        assert!(err.contains("`ld.fs.write` failed to create"));
    }
}
