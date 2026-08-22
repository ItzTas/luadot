use std::path::Path;

use mlua::{Function, Lua, Value};

use super::constants::LS;
use super::resolve::{failed, resolve};

pub fn function(lua: &Lua) -> mlua::Result<Function> {
    lua.create_function(|lua, value: Value| {
        let path = resolve(lua, &value, LS)?;

        lua.create_sequence_from(names(&path)?)
    })
}

fn names(dir: &Path) -> mlua::Result<Vec<String>> {
    let entries = std::fs::read_dir(dir).map_err(|err| failed(LS, "read", dir.display(), err))?;

    let mut names = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|err| failed(LS, "read", dir.display(), err))?;
        names.push(entry.file_name().to_string_lossy().into_owned());
    }
    names.sort();

    Ok(names)
}

#[cfg(test)]
mod tests {
    use super::super::fixture::eval;

    #[test]
    fn lists_files_and_directories_by_name_in_order() {
        let home = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(home.path().join("plugins/b")).unwrap();
        std::fs::create_dir_all(home.path().join("plugins/a")).unwrap();
        std::fs::write(home.path().join("plugins/lock.json"), "{}").unwrap();

        assert_eq!(
            eval(home.path(), r#"return table.concat(fs.ls("plugins"), ",")"#).unwrap(),
            "a,b,lock.json"
        );
    }

    #[test]
    fn a_missing_directory_is_reported() {
        let home = tempfile::tempdir().unwrap();

        let err = eval(home.path(), r#"return fs.ls("missing")"#)
            .unwrap_err()
            .to_string();

        assert!(err.contains("`ld.fs.ls` failed to read"));
    }
}
