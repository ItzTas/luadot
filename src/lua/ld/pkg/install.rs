use std::env;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::process::Command;

use mlua::{Function, Lua, Value};

use super::super::constants::API;
use super::super::parse::external;
use super::super::surface;
use super::constants::{INSTALL, MANAGERS, NAMESPACE, SUDO};

pub fn function(lua: &Lua) -> mlua::Result<Function> {
    lua.create_function(|lua, packages: Value| {
        surface::slow(lua, &format!("{NAMESPACE}.{INSTALL}"));
        install(&packages)
    })
}

fn install(value: &Value) -> mlua::Result<()> {
    let packages = packages(value)?;
    if packages.is_empty() {
        return Ok(());
    }

    let path = env::var_os("PATH");
    let (manager, args) = detect(path.as_deref()).ok_or_else(|| {
        external(format!(
            "{} found no supported package manager (supported: {})",
            prefix(),
            names()
        ))
    })?;
    let sudo = find_in_path(SUDO, path.as_deref()).is_some();

    run(build_command(manager, args, sudo, &packages), manager)
}

fn run(mut command: Command, manager: &str) -> mlua::Result<()> {
    let status = command
        .status()
        .map_err(|err| external(format!("{} failed to run `{manager}`: {err}", prefix())))?;

    if status.success() {
        return Ok(());
    }

    let code = status
        .code()
        .map_or_else(|| "signal".to_string(), |code| code.to_string());
    Err(external(format!(
        "{} `{manager}` exited with status {code}",
        prefix()
    )))
}

fn build_command(manager: &str, args: &[&str], sudo: bool, packages: &[String]) -> Command {
    let mut command = base_command(manager, sudo);
    command.args(args);
    command.args(packages);
    command
}

fn base_command(manager: &str, sudo: bool) -> Command {
    if !sudo {
        return Command::new(manager);
    }

    let mut command = Command::new(SUDO);
    command.arg(manager);
    command
}

fn packages(value: &Value) -> mlua::Result<Vec<String>> {
    match value {
        Value::String(name) => Ok(vec![name.to_str()?.to_string()]),
        Value::Table(list) => list
            .clone()
            .sequence_values::<String>()
            .collect::<mlua::Result<Vec<_>>>()
            .map_err(|_| invalid()),
        _ => Err(invalid()),
    }
}

fn detect(path: Option<&OsStr>) -> Option<(&'static str, &'static [&'static str])> {
    MANAGERS
        .iter()
        .find(|(manager, _)| find_in_path(manager, path).is_some())
        .map(|(manager, args)| (*manager, *args))
}

fn find_in_path(program: &str, path: Option<&OsStr>) -> Option<PathBuf> {
    env::split_paths(path?)
        .map(|dir| dir.join(program))
        .find(|candidate| candidate.is_file())
}

fn invalid() -> mlua::Error {
    external(format!(
        "{} takes a package name or a list of package names",
        prefix()
    ))
}

fn prefix() -> String {
    format!("`{API}.{NAMESPACE}.{INSTALL}`")
}

fn names() -> String {
    MANAGERS
        .iter()
        .map(|(name, _)| *name)
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use std::ffi::{OsStr, OsString};

    use super::*;

    fn path_with(programs: &[&str]) -> (tempfile::TempDir, OsString) {
        let dir = tempfile::tempdir().unwrap();
        for program in programs {
            std::fs::write(dir.path().join(program), "").unwrap();
        }
        let path = env::join_paths([dir.path()]).unwrap();
        (dir, path)
    }

    #[test]
    fn detect_prefers_managers_in_declaration_order() {
        let (_dir, path) = path_with(&["dnf", "pacman"]);

        let (manager, _) = detect(Some(&path)).unwrap();

        assert_eq!(manager, "pacman");
    }

    #[test]
    fn build_command_prefixes_sudo_when_available() {
        let command = build_command("pacman", &["-S"], true, &["git".to_string()]);

        assert_eq!(command.get_program(), OsStr::new("sudo"));
        let args: Vec<&str> = command.get_args().map(|a| a.to_str().unwrap()).collect();
        assert_eq!(args, ["pacman", "-S", "git"]);
    }
}
