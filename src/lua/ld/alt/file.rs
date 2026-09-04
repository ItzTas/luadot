use std::fmt::Display;
use std::path::{Path, PathBuf};

use mlua::{AnyUserData, Function, Lua};

use super::super::constants::API;
use super::super::parse::external;
use super::constants::{FILE, NAMESPACE};
use crate::lua::{Handle, Scope};

pub fn function(lua: &Lua) -> mlua::Result<Function> {
    lua.create_function(|lua, name: String| {
        let path = resolve(lua, &name, FILE)?;
        lua.create_userdata(Handle::new(path))
    })
}

pub fn resolve(lua: &Lua, name: &str, caller: &str) -> mlua::Result<PathBuf> {
    let (dir, path) = {
        let scope = Scope::building(lua)?;
        (scope.dir().to_path_buf(), scope.resolve(name))
    };

    path.ok_or_else(|| external(missing(name, caller, &dir)))
}

fn missing(name: &str, caller: &str, dir: &Path) -> String {
    let call = format!("`{API}.{NAMESPACE}.{caller}`");
    if Path::new(name).is_absolute() {
        return format!("{call} found no file at {name}");
    }

    format!("{call} found no file `{name}` in {}", dir.display())
}

pub fn failed(caller: &str, action: &str, subject: impl Display, err: impl Display) -> mlua::Error {
    external(format!(
        "`{API}.{NAMESPACE}.{caller}` failed to {action} {subject}: {err}"
    ))
}

pub fn read(caller: &str, path: &Path) -> mlua::Result<String> {
    std::fs::read_to_string(path).map_err(|err| failed(caller, "read", path.display(), err))
}

pub fn handle(data: &AnyUserData) -> mlua::Result<PathBuf> {
    data.borrow::<Handle>()
        .map(|handle| handle.path().to_path_buf())
        .map_err(|_| {
            external(format!(
                "`{API}.{NAMESPACE}.{FILE}` was given something else"
            ))
        })
}

#[cfg(test)]
mod tests {
    use super::super::fixture::template;
    use crate::lua::{Content, from_template};

    #[test]
    fn reaches_outside_the_template() {
        let root = tempfile::tempdir().unwrap();
        let dir = template(root.path());
        let outside = root.path().join("outside.zsh");
        std::fs::write(&outside, "outside").unwrap();

        let relative = from_template(&dir, r#"return ld.alt.file("../outside.zsh")"#).unwrap();
        let absolute = from_template(
            &dir,
            &format!(r#"return ld.alt.file("{}")"#, outside.display()),
        )
        .unwrap();

        assert_eq!(absolute[0].content(), &Content::File(outside));
        assert_eq!(
            relative[0].content(),
            &Content::File(dir.join("../outside.zsh"))
        );
    }
}
