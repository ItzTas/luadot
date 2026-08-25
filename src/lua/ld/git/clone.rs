use std::num::NonZeroU32;

use mlua::{Function, Lua, Table, Value};

use super::super::constants::API;
use super::super::parse::{chain, external, known};
use super::super::surface;
use super::super::value::{count, path, text};
use super::constants::{BRANCH, CLONE, CLONE_KEYS, DEPTH, NAMESPACE};
use crate::git;
use crate::lua::Scope;
use crate::utils::expand;

pub fn function(lua: &Lua) -> mlua::Result<Function> {
    lua.create_function(|lua, (url, dir, options): (Value, Value, Option<Table>)| {
        let call = format!("{NAMESPACE}.{CLONE}");
        surface::slow(lua, &call);

        let url = text(NAMESPACE, &url, CLONE)?;
        let raw = path(NAMESPACE, &dir, CLONE, "a directory")?;
        let home = Scope::building(lua)?.home().to_path_buf();
        let dir = expand(&home, &raw);
        let (branch, depth) = parse(&call, options)?;

        git::clone_plain(
            &format!("`{API}.{call}`"),
            &dir,
            &url,
            branch.as_deref(),
            depth,
        )
        .map_err(chain)
    })
}

fn parse(call: &str, options: Option<Table>) -> mlua::Result<(Option<String>, Option<NonZeroU32>)> {
    let Some(options) = options else {
        return Ok((None, None));
    };
    known(call, &options, &CLONE_KEYS)?;

    let branch = match options.get::<Value>(BRANCH)? {
        Value::Nil => None,
        value => Some(text(call, &value, BRANCH)?),
    };
    let depth = match options.get::<Value>(DEPTH)? {
        Value::Nil => None,
        value => Some(depth(call, &value)?),
    };

    Ok((branch, depth))
}

fn depth(call: &str, value: &Value) -> mlua::Result<NonZeroU32> {
    NonZeroU32::new(count(call, value, DEPTH)?).ok_or_else(|| {
        external(format!(
            "`{API}.{call}.{DEPTH}` takes a whole number of one or more"
        ))
    })
}

#[cfg(test)]
mod tests {
    use super::super::fixture::eval;
    use crate::git::fixture::{repository, stage};

    fn committed() -> tempfile::TempDir {
        let origin = repository();
        stage(origin.path(), "tracked");
        let status = std::process::Command::new("git")
            .current_dir(origin.path())
            .args(["commit", "--quiet", "-m", "first"])
            .status()
            .unwrap();
        assert!(status.success());

        origin
    }

    #[test]
    fn clones_into_a_directory_under_the_home_directory() {
        let origin = committed();
        let home = tempfile::tempdir().unwrap();

        eval(
            home.path(),
            &format!(
                r#"git.clone("{}", "plugins/tracked", {{ depth = 1 }}) return "done""#,
                origin.path().display()
            ),
        )
        .unwrap();

        assert_eq!(
            std::fs::read_to_string(home.path().join("plugins/tracked/tracked")).unwrap(),
            "contents\n"
        );
    }

    #[test]
    fn rejects_an_unknown_option_and_a_depth_of_nothing() {
        let home = tempfile::tempdir().unwrap();

        let err = eval(
            home.path(),
            r#"git.clone("url", "dir", { branhc = "main" })"#,
        )
        .unwrap_err()
        .to_string();
        assert!(err.contains("`ld.git.clone`: unknown key `branhc`"));
        assert!(err.contains("available: branch, depth"));

        let err = eval(home.path(), r#"git.clone("url", "dir", { depth = 0 })"#)
            .unwrap_err()
            .to_string();
        assert!(err.contains("`ld.git.clone.depth` takes a whole number of one or more"));
    }
}
