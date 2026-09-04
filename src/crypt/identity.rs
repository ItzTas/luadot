use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};

use super::constants::{SECRET_MODE, SHELL, SHELL_ARG};
use super::edit::Workspace;
use crate::files::write_mode;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Provider {
    Line(String),
    Program(Vec<String>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Key {
    File(PathBuf),
    Command(Provider),
}

#[derive(Debug, Default)]
pub struct Identity {
    key: Option<Key>,
    workspace: Option<Workspace>,
    provided: Option<PathBuf>,
}

impl Provider {
    pub fn command(&self) -> Command {
        match self {
            Self::Line(line) => {
                let mut command = Command::new(SHELL);
                command.arg(SHELL_ARG).arg(line);
                command
            }
            Self::Program(words) => {
                let mut command = Command::new(&words[0]);
                command.args(&words[1..]);
                command
            }
        }
    }

    pub fn label(&self) -> String {
        match self {
            Self::Line(line) => line.clone(),
            Self::Program(words) => words.join(" "),
        }
    }
}

impl Identity {
    pub fn new(key: Option<Key>) -> Self {
        Self {
            key,
            workspace: None,
            provided: None,
        }
    }

    pub fn path(&mut self, command: &str) -> Result<Option<&Path>> {
        let provider = match &self.key {
            None => return Ok(None),
            Some(Key::File(file)) => return Ok(Some(file)),
            Some(Key::Command(provider)) => provider,
        };
        if self.provided.is_none() {
            let (workspace, path) = provide(command, provider)?;
            self.workspace = Some(workspace);
            self.provided = Some(path);
        }

        Ok(self.provided.as_deref())
    }
}

fn provide(command: &str, provider: &Provider) -> Result<(Workspace, PathBuf)> {
    let label = provider.label();
    let output = provider
        .command()
        .output()
        .with_context(|| format!("{command}: failed to run `{label}`"))?;

    if !output.status.success() {
        bail!(
            "{command}: `{label}` failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    if output.stdout.iter().all(u8::is_ascii_whitespace) {
        bail!("{command}: `{label}` produced no identity");
    }

    let workspace = Workspace::create(command)?;
    let path = workspace.file(OsStr::new("identity"));
    write_mode(command, &path, &output.stdout, SECRET_MODE)?;

    Ok((workspace, path))
}

#[cfg(test)]
mod tests {
    use std::os::unix::fs::PermissionsExt;

    use super::*;

    #[test]
    fn a_provided_identity_is_private() {
        let mut identity = Identity::new(Some(Key::Command(Provider::Line(
            "printf 'AGE-SECRET-KEY-1TEST\n'".to_string(),
        ))));

        let path = identity.path("apply").unwrap().unwrap().to_path_buf();

        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            "AGE-SECRET-KEY-1TEST\n"
        );
        assert_eq!(
            std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            SECRET_MODE
        );
    }

    #[test]
    fn the_provider_runs_once() {
        let dir = tempfile::tempdir().unwrap();
        let counter = dir.path().join("runs");
        let mut identity = Identity::new(Some(Key::Command(Provider::Line(format!(
            "printf x >> {}; printf 'AGE-SECRET-KEY-1TEST\n'",
            counter.display()
        )))));

        let first = identity.path("apply").unwrap().unwrap().to_path_buf();
        let second = identity.path("apply").unwrap().unwrap().to_path_buf();

        assert_eq!(first, second);
        assert_eq!(std::fs::read_to_string(&counter).unwrap(), "x");
    }

    #[test]
    fn dropping_it_removes_the_file() {
        let path = {
            let mut identity = Identity::new(Some(Key::Command(Provider::Line(
                "printf 'AGE-SECRET-KEY-1TEST\n'".to_string(),
            ))));
            identity.path("apply").unwrap().unwrap().to_path_buf()
        };

        assert!(std::fs::symlink_metadata(&path).is_err());
    }
}
