use std::env;
use std::ffi::OsString;
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result};

use super::constants::DEFAULT_EDITOR;

pub fn open(command: &str, path: &Path) -> Result<()> {
    let status = launch(command, path)?;

    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }

    Ok(())
}

pub fn launch(command: &str, path: &Path) -> Result<std::process::ExitStatus> {
    let editor = resolve_editor(env::var_os("VISUAL"), env::var_os("EDITOR"));
    build_command(&editor, path)
        .status()
        .with_context(|| format!("{command}: failed to launch editor `{editor}`"))
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

#[cfg(test)]
mod tests {
    use std::ffi::OsStr;
    use std::path::Path;

    use super::build_command;

    #[test]
    fn build_command_forwards_editor_arguments() {
        let command = build_command("code -w", Path::new("/repo/.bashrc"));

        assert_eq!(command.get_program(), OsStr::new("code"));
        let args: Vec<&OsStr> = command.get_args().collect();
        assert_eq!(args, [OsStr::new("-w"), OsStr::new("/repo/.bashrc")]);
    }
}
