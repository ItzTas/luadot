use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};

use crate::state;

pub fn git_cmd(args: &[String]) -> Result<()> {
    let repo = repo_dir()?;
    if !repo.is_dir() {
        bail!(
            "git: repository {} does not exist; run `luadot clone <url>` first",
            repo.display()
        );
    }

    let status = build_command(&repo, args)
        .status()
        .context("git: failed to run git; is it installed and on PATH?")?;

    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }

    Ok(())
}

fn build_command(repo: &Path, args: &[String]) -> Command {
    let mut command = Command::new("git");
    command.current_dir(repo);
    command.args(args);
    command
}

fn repo_dir() -> Result<PathBuf> {
    let state = state::load()?;
    state
        .repo
        .context("git: no repository set; run `luadot clone <url>` first")
}

#[cfg(test)]
mod tests {
    use std::ffi::OsStr;
    use std::path::Path;

    use super::build_command;

    #[test]
    fn runs_git_in_repo_dir_forwarding_all_args() {
        let command = build_command(
            Path::new("/tmp/luadot-repo"),
            &["commit".to_string(), "-m".to_string(), "msg".to_string()],
        );

        assert_eq!(command.get_program(), OsStr::new("git"));
        assert_eq!(command.get_current_dir(), Some(Path::new("/tmp/luadot-repo")));

        let args: Vec<&str> = command.get_args().map(|a| a.to_str().unwrap()).collect();
        assert_eq!(args, ["commit", "-m", "msg"]);
    }
}
