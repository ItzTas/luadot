use std::process::{Command, ExitStatus, Stdio};

use mlua::Lua;

use super::constants::API;
use super::parse::external;
use super::surface;

pub fn run(
    lua: &Lua,
    mut command: Command,
    namespace: &str,
    display: &str,
) -> mlua::Result<String> {
    surface::slow(lua, namespace);

    let output = command
        .stdin(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .map_err(|err| {
            external(format!(
                "{} failed to run `{display}`: {err}",
                prefix(namespace)
            ))
        })?;

    if !output.status.success() {
        return Err(failed(&output.status, namespace, display));
    }

    let stdout = String::from_utf8(output.stdout).map_err(|_| {
        external(format!(
            "{} `{display}` produced invalid UTF-8",
            prefix(namespace)
        ))
    })?;

    Ok(stdout.trim_end().to_string())
}

pub fn display(name: &str, args: &[String]) -> String {
    std::iter::once(name)
        .chain(args.iter().map(String::as_str))
        .collect::<Vec<_>>()
        .join(" ")
}

fn failed(status: &ExitStatus, namespace: &str, display: &str) -> mlua::Error {
    let code = status
        .code()
        .map_or_else(|| "signal".to_string(), |code| code.to_string());

    external(format!(
        "{} `{display}` exited with status {code}",
        prefix(namespace)
    ))
}

fn prefix(namespace: &str) -> String {
    format!("`{API}.{namespace}`")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lua::runtime::runtime;

    fn shell(line: &str) -> Command {
        let mut command = Command::new("sh");
        command.arg("-c").arg(line);
        command
    }

    #[test]
    fn yields_the_output_without_its_trailing_newline() {
        let lua = runtime().unwrap();

        let output = run(&lua, shell("printf 'one\ntwo\n'"), "cmd", "test").unwrap();

        assert_eq!(output, "one\ntwo");
    }

    #[test]
    fn reports_the_status_of_a_failing_command() {
        let lua = runtime().unwrap();

        let err = run(&lua, shell("exit 3"), "cmd", "test")
            .unwrap_err()
            .to_string();

        assert!(err.contains("`ld.cmd` `test` exited with status 3"));
    }

    #[test]
    fn the_message_names_the_namespace_that_ran_it() {
        let lua = runtime().unwrap();

        let err = run(&lua, shell("exit 1"), "git", "git status")
            .unwrap_err()
            .to_string();

        assert!(err.contains("`ld.git` `git status` exited with status 1"));
    }

    #[test]
    fn reports_a_command_that_cannot_be_started() {
        let lua = runtime().unwrap();

        let err = run(&lua, Command::new("luadot-no-such-program"), "cmd", "test")
            .unwrap_err()
            .to_string();

        assert!(err.contains("`ld.cmd` failed to run `test`"));
    }

    #[test]
    fn reports_output_that_is_not_utf8() {
        let lua = runtime().unwrap();

        let err = run(&lua, shell(r"printf '\377'"), "cmd", "test")
            .unwrap_err()
            .to_string();

        assert!(err.contains("produced invalid UTF-8"));
    }

    #[test]
    fn the_display_names_the_whole_invocation() {
        assert_eq!(
            display("git", &["clone".to_string(), "url".to_string()]),
            "git clone url"
        );
        assert_eq!(display("ls", &[]), "ls");
    }
}
