use std::path::{Path, PathBuf};

use mlua::{AnyUserData, Function, Lua, Value};

use super::super::constants::API;
use super::super::parse::external;
use super::super::surface::{self, Surface};
use super::constants::{FILE, NAMESPACE};
use crate::lua::{Handle, Template};

pub fn function(lua: &Lua) -> mlua::Result<Function> {
    lua.create_function(|lua, name: String| {
        if surface::inert(lua, &format!("{NAMESPACE}.{FILE}"), Surface::Template) {
            return Ok(Value::Nil);
        }

        let path = resolve(lua, &name, FILE)?;
        Ok(Value::UserData(lua.create_userdata(Handle::new(path))?))
    })
}

pub fn resolve(lua: &Lua, name: &str, caller: &str) -> mlua::Result<PathBuf> {
    let (dir, path) = {
        let template = Template::building(lua)?;
        (template.dir().to_path_buf(), template.resolve(name))
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
    use std::path::{Path, PathBuf};

    use crate::lua::{Content, from_template};

    fn template(root: &Path) -> PathBuf {
        let dir = root.join(".zshrc.luadot");
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn reports_a_file_the_template_does_not_hold() {
        let root = tempfile::tempdir().unwrap();
        let dir = template(root.path());

        let err = format!(
            "{:#}",
            from_template(&dir, r#"return ld.alt.file("missing.zsh")"#).unwrap_err()
        );

        assert!(err.contains("found no file `missing.zsh`"));
        assert!(err.contains(&dir.display().to_string()));
    }

    #[test]
    fn reaches_a_file_outside_the_template_directory() {
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

    #[test]
    fn reports_an_absolute_path_that_holds_no_file() {
        let root = tempfile::tempdir().unwrap();
        let dir = template(root.path());

        let err = format!(
            "{:#}",
            from_template(&dir, r#"return ld.alt.file("/nowhere/missing.zsh")"#).unwrap_err()
        );

        assert!(err.contains("found no file at /nowhere/missing.zsh"));
    }
}
