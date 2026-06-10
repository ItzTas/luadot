use std::env;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};

use crate::{state, utils};

const DEFAULT_EDITOR: &str = "vi";

pub fn edit_cmd(args: &[String]) -> Result<()> {
    let target = args.first().context("edit: missing file path")?;
    let target =
        std::path::absolute(target).with_context(|| format!("edit: invalid path {target}"))?;

    let repo = repo_dir()?;
    if !repo.is_dir() {
        bail!(
            "edit: repository {} does not exist; run `luadot clone <url>` first",
            repo.display()
        );
    }

    let home = utils::home_dir()?;
    let in_repo = utils::repo_path(&home, &repo, &target)?;
    if !in_repo.exists() {
        bail!(
            "edit: {} is not managed by the repository; run `luadot add` first",
            target.display()
        );
    }

    // The default editor comes from `$VISUAL`/`$EDITOR`; a future Lua
    // configuration will allow overriding it.
    let editor = resolve_editor(env::var_os("VISUAL"), env::var_os("EDITOR"));
    let status = build_command(&editor, &in_repo)
        .status()
        .with_context(|| format!("edit: failed to launch editor `{editor}`"))?;

    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }

    Ok(())
}

fn resolve_editor(visual: Option<OsString>, editor: Option<OsString>) -> String {
    [visual, editor]
        .into_iter()
        .flatten()
        .find(|value| !value.is_empty())
        .map(|value| value.to_string_lossy().into_owned())
        .unwrap_or_else(|| DEFAULT_EDITOR.to_string())
}

fn build_command(editor: &str, path: &Path) -> Command {
    let mut parts = editor.split_whitespace();
    let program = parts.next().unwrap_or(DEFAULT_EDITOR);
    let mut command = Command::new(program);
    command.args(parts);
    command.arg(path);
    command
}

fn repo_dir() -> Result<PathBuf> {
    let state = state::load()?;
    state
        .repo
        .context("edit: no repository set; run `luadot clone <url>` first")
}

#[cfg(test)]
mod tests {
    use std::ffi::{OsStr, OsString};
    use std::path::Path;

    use super::{DEFAULT_EDITOR, build_command, resolve_editor};

    #[test]
    fn resolve_editor_prefers_visual() {
        assert_eq!(resolve_editor(Some("nvim".into()), Some("vi".into())), "nvim");
    }

    #[test]
    fn resolve_editor_falls_back_to_editor_when_visual_is_unset_or_empty() {
        assert_eq!(resolve_editor(None, Some("vi".into())), "vi");
        assert_eq!(resolve_editor(Some(OsString::new()), Some("vi".into())), "vi");
    }

    #[test]
    fn resolve_editor_defaults_when_nothing_is_set() {
        assert_eq!(resolve_editor(None, None), DEFAULT_EDITOR);
    }

    #[test]
    fn build_command_appends_the_path_to_a_bare_editor() {
        let command = build_command("vim", Path::new("/repo/.bashrc"));

        assert_eq!(command.get_program(), OsStr::new("vim"));
        let args: Vec<&OsStr> = command.get_args().collect();
        assert_eq!(args, [OsStr::new("/repo/.bashrc")]);
    }

    #[test]
    fn build_command_forwards_editor_arguments() {
        let command = build_command("code -w", Path::new("/repo/.bashrc"));

        assert_eq!(command.get_program(), OsStr::new("code"));
        let args: Vec<&OsStr> = command.get_args().collect();
        assert_eq!(args, [OsStr::new("-w"), OsStr::new("/repo/.bashrc")]);
    }
}
