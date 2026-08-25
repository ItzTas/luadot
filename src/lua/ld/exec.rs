use std::process::{Command, ExitStatus, Stdio};

use super::constants::API;
use super::parse::external;

pub fn run(mut command: Command, namespace: &str, display: &str) -> mlua::Result<String> {
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

    fn shell(line: &str) -> Command {
        let mut command = Command::new("sh");
        command.arg("-c").arg(line);
        command
    }

    #[test]
    fn trims_the_trailing_newline() {
        let output = run(shell("printf 'one\ntwo\n'"), "cmd", "test").unwrap();

        assert_eq!(output, "one\ntwo");
    }

    #[test]
    fn reports_a_failing_status() {
        let err = run(shell("exit 3"), "cmd", "test").unwrap_err().to_string();

        assert!(err.contains("`ld.cmd` `test` exited with status 3"));
    }
}
