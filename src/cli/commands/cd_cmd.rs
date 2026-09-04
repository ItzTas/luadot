use std::env;
use std::ffi::OsString;
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result};

use crate::{lua, utils};

pub fn cd_cmd() -> Result<()> {
    let config = lua::load_config()?;
    let repo = utils::require_repo("cd", utils::configured("cd", &config)?.repo_dir())?;

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
        .unwrap_or_else(|| "/bin/sh".to_string())
}

fn build_command(shell: &str, repo: &Path) -> Command {
    let mut command = Command::new(shell);
    command.current_dir(repo);
    command
}
