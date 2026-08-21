use std::io::ErrorKind;
use std::path::Path;

use mlua::{Function, Lua, Value};

use super::super::constants::API;
use super::super::parse::external;
use super::constants::{NAMESPACE, RM};
use super::resolve::{failed, resolve};
use crate::lua::Scope;

pub fn function(lua: &Lua) -> mlua::Result<Function> {
    lua.create_function(|lua, value: Value| {
        let path = resolve(lua, &value, RM)?;
        let home = Scope::building(lua)?.home().to_path_buf();
        if home.starts_with(&path) {
            return Err(external(format!(
                "`{API}.{NAMESPACE}.{RM}` refuses to remove {}: it holds your home directory",
                path.display()
            )));
        }

        remove(&path)
    })
}

fn remove(path: &Path) -> mlua::Result<bool> {
    let metadata = match std::fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(err) if err.kind() == ErrorKind::NotFound => return Ok(false),
        Err(err) => return Err(failed(RM, "inspect", path.display(), err)),
    };

    let removed = match metadata.is_dir() {
        true => std::fs::remove_dir_all(path),
        false => std::fs::remove_file(path),
    };
    removed.map_err(|err| failed(RM, "remove", path.display(), err))?;

    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::super::fixture::eval;

    #[test]
    fn removes_a_file_a_symlink_or_a_whole_directory_and_says_whether_it_was_there() {
        let home = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(home.path().join("plugins/old/lua")).unwrap();
        std::fs::write(home.path().join("plugins/old/lua/old.lua"), "").unwrap();
        std::fs::write(home.path().join("lock.json"), "{}").unwrap();
        std::os::unix::fs::symlink(home.path().join("lock.json"), home.path().join("link"))
            .unwrap();

        assert_eq!(
            eval(
                home.path(),
                r#"return tostring(fs.rm("plugins/old")) .. "/" .. tostring(fs.rm("link")) .. "/" .. tostring(fs.rm("lock.json")) .. "/" .. tostring(fs.rm("lock.json"))"#
            )
            .unwrap(),
            "true/true/true/false"
        );
        assert!(!home.path().join("plugins/old").exists());
        assert!(!home.path().join("link").exists());
        assert!(!home.path().join("lock.json").exists());
    }

    #[test]
    fn the_home_directory_and_what_holds_it_are_refused() {
        let home = tempfile::tempdir().unwrap();

        for path in ["~", "/"] {
            let err = eval(home.path(), &format!(r#"return fs.rm("{path}")"#))
                .unwrap_err()
                .to_string();

            assert!(err.contains("`ld.fs.rm` refuses to remove"), "{path}");
        }
        assert!(home.path().is_dir());
    }
}
