use std::env;
use std::ffi::OsString;
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result};

use crate::utils;

const DEFAULT_SHELL: &str = "/bin/sh";

pub fn cd_cmd() -> Result<()> {
    let repo = utils::require_repo("cd")?;

    let shell = resolve_shell(env::var_os("SHELL"));
    let status = build_command(&shell, &repo)
        .status()
        .with_context(|| format!("cd: failed to launch shell `{shell}`"))?;

    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }

    Ok(())
}

fn resolve_shell(shell: Option<OsString>) -> String {
    shell
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string_lossy().into_owned())
        .unwrap_or_else(|| DEFAULT_SHELL.to_string())
}

fn build_command(shell: &str, repo: &Path) -> Command {
    let mut command = Command::new(shell);
    command.current_dir(repo);
    command
}

#[cfg(test)]
mod tests {
    use std::ffi::{OsStr, OsString};
    use std::path::Path;

    use super::{DEFAULT_SHELL, build_command, resolve_shell};

    #[test]
    fn resolve_shell_uses_the_environment_value() {
        assert_eq!(resolve_shell(Some("/usr/bin/zsh".into())), "/usr/bin/zsh");
    }

    #[test]
    fn resolve_shell_defaults_when_unset_or_empty() {
        assert_eq!(resolve_shell(None), DEFAULT_SHELL);
        assert_eq!(resolve_shell(Some(OsString::new())), DEFAULT_SHELL);
    }

    #[test]
    fn build_command_starts_the_shell_in_the_repo_dir() {
        let command = build_command("/usr/bin/zsh", Path::new("/tmp/luadot-repo"));

        assert_eq!(command.get_program(), OsStr::new("/usr/bin/zsh"));
        assert_eq!(
            command.get_current_dir(),
            Some(Path::new("/tmp/luadot-repo"))
        );
        assert_eq!(command.get_args().count(), 0);
    }
}
