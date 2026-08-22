use std::path::Path;
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
        let repo = require(paths.repo(), &command)?;

        run_in(lua, repo, &args, &command)
    })
}

pub fn run_in(lua: &Lua, dir: &Path, args: &[String], command: &str) -> mlua::Result<String> {
    if args.is_empty() {
        return Err(external(format!(
            "{command} takes the arguments of the git command to run"
        )));
    }

    let mut git = Command::new(PROGRAM);
    git.current_dir(dir).args(args);

    run(lua, git, NAMESPACE, &display(PROGRAM, args))
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::super::super::fixture;
    use super::super::super::path::Paths;
    use super::super::constants::NAMESPACE;
    use super::super::table::table;
    use crate::git::fixture::repository;

    fn eval(paths: &Paths, source: &str) -> mlua::Result<String> {
        fixture::eval(NAMESPACE, |lua| table(lua, paths), source)
    }

    fn paths(repo: Option<&Path>) -> Paths {
        Paths::new(
            Path::new("/home/u"),
            Path::new("/home/u/.config/luadot"),
            Path::new("/home/u/.local/share/luadot"),
        )
        .with_repo(repo)
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
}
