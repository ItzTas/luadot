use std::process::Command;

use mlua::{Function, Lua, Table, Variadic};

use super::super::constants::API;
use super::super::exec::{display, run};
use super::super::parse::external;
use super::super::path::Paths;
use super::super::repo::require;
use super::constants::{NAMESPACE, PROGRAM};

pub fn function(lua: &Lua, paths: &Paths) -> mlua::Result<Function> {
    let paths = paths.clone();
    let command = format!("`{API}.{NAMESPACE}`");

    lua.create_function(move |lua, (_, args): (Table, Variadic<String>)| {
        if args.is_empty() {
            return Err(external(format!(
                "{command} takes the arguments of the git command to run"
            )));
        }

        let repo = require(paths.repo(), &command)?;
        let mut git = Command::new(PROGRAM);
        git.current_dir(repo).args(args.iter());

        run(lua, git, NAMESPACE, &display(PROGRAM, &args))
    })
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::process::Command;

    use tempfile::TempDir;

    use super::super::super::path::Paths;
    use super::super::table::table;
    use crate::lua::runtime::runtime;

    fn eval(paths: &Paths, source: &str) -> mlua::Result<String> {
        let lua = runtime().unwrap();
        lua.globals()
            .set("git", table(&lua, paths).unwrap())
            .unwrap();

        lua.load(source).eval()
    }

    fn paths(repo: Option<&Path>) -> Paths {
        Paths::new(Path::new("/home/u"), Path::new("/home/u/.config/luadot")).with_repo(repo)
    }

    fn repository() -> TempDir {
        let repo = tempfile::tempdir().unwrap();

        for args in [
            vec!["init", "--quiet"],
            vec!["config", "user.email", "test@luadot"],
            vec!["config", "user.name", "luadot"],
        ] {
            let status = Command::new("git")
                .current_dir(repo.path())
                .args(args)
                .status()
                .unwrap();
            assert!(status.success());
        }

        repo
    }

    #[test]
    fn runs_git_inside_the_repository() {
        let repo = repository();
        std::fs::write(repo.path().join("tracked"), "one\n").unwrap();

        let output = eval(
            &paths(Some(repo.path())),
            r#"return git("status", "--porcelain")"#,
        )
        .unwrap();

        assert_eq!(output, "?? tracked");
    }

    #[test]
    fn every_argument_stays_literal() {
        let repo = repository();
        std::fs::write(repo.path().join("tracked"), "one\n").unwrap();
        let paths = paths(Some(repo.path()));

        eval(&paths, r#"return git("add", "tracked")"#).unwrap();
        eval(&paths, r#"return git("commit", "-m", "one  two")"#).unwrap();

        assert_eq!(
            eval(&paths, r#"return git("log", "-1", "--format=%s")"#).unwrap(),
            "one  two"
        );
    }

    #[test]
    fn a_failing_command_stops_the_script() {
        let repo = repository();

        let err = eval(
            &paths(Some(repo.path())),
            r#"return git("no-such-command")"#,
        )
        .unwrap_err()
        .to_string();

        assert!(err.contains("`ld.git` `git no-such-command` exited with status"));
    }

    #[test]
    fn reports_a_call_without_arguments() {
        let repo = repository();

        let err = eval(&paths(Some(repo.path())), "return git()")
            .unwrap_err()
            .to_string();

        assert!(err.contains("`ld.git` takes the arguments of the git command to run"));
    }

    #[test]
    fn reports_a_missing_repository() {
        let err = eval(&paths(None), r#"return git("status")"#)
            .unwrap_err()
            .to_string();

        assert!(err.contains("`ld.git`: no repository set"));
    }
}
