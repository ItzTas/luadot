use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};

use super::constants::{IDENTITY_FILE, SECRET_MODE, SHELL, SHELL_ARG};
use super::edit::Workspace;
use crate::files::write_mode;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Provider {
    Line(String),
    Program(Vec<String>),
}

#[derive(Debug, Default)]
pub struct Identity {
    file: Option<PathBuf>,
    provider: Option<Provider>,
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
    pub fn new(file: Option<PathBuf>, provider: Option<Provider>) -> Self {
        Self {
            file,
            provider,
            workspace: None,
            provided: None,
        }
    }

    pub fn path(&mut self, command: &str) -> Result<Option<&Path>> {
        let Some(provider) = &self.provider else {
            return Ok(self.file.as_deref());
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
    let path = workspace.file(OsStr::new(IDENTITY_FILE));
    write_mode(command, &path, &output.stdout, SECRET_MODE)?;

    Ok((workspace, path))
}

#[cfg(test)]
mod tests {
    use std::os::unix::fs::PermissionsExt;

    use super::*;

    fn program(words: &[&str]) -> Provider {
        Provider::Program(words.iter().map(|word| word.to_string()).collect())
    }

    #[test]
    fn a_line_runs_through_the_shell() {
        let command = Provider::Line("pass show age/key".to_string()).command();

        assert_eq!(command.get_program(), OsStr::new("sh"));
        assert_eq!(
            command
                .get_args()
                .map(|arg| arg.to_string_lossy().into_owned())
                .collect::<Vec<String>>(),
            ["-c", "pass show age/key"]
        );
    }

    #[test]
    fn a_list_runs_the_program_it_names() {
        let command = program(&["op", "read", "op://vault/age/key"]).command();

        assert_eq!(command.get_program(), OsStr::new("op"));
        assert_eq!(
            command
                .get_args()
                .map(|arg| arg.to_string_lossy().into_owned())
                .collect::<Vec<String>>(),
            ["read", "op://vault/age/key"]
        );
    }

    #[test]
    fn without_a_provider_the_identity_is_the_configured_file() {
        let mut identity = Identity::new(Some(PathBuf::from("/home/u/key.txt")), None);

        assert_eq!(
            identity.path("apply").unwrap(),
            Some(Path::new("/home/u/key.txt"))
        );
    }

    #[test]
    fn without_anything_there_is_no_identity() {
        assert_eq!(Identity::default().path("apply").unwrap(), None);
    }

    #[test]
    fn the_provider_wins_over_the_file_and_lands_in_a_private_file() {
        let mut identity = Identity::new(
            Some(PathBuf::from("/home/u/key.txt")),
            Some(Provider::Line(
                "printf 'AGE-SECRET-KEY-1TEST\n'".to_string(),
            )),
        );

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
    fn the_provider_runs_once_for_the_whole_command() {
        let dir = tempfile::tempdir().unwrap();
        let counter = dir.path().join("runs");
        let mut identity = Identity::new(
            None,
            Some(Provider::Line(format!(
                "printf x >> {}; printf 'AGE-SECRET-KEY-1TEST\n'",
                counter.display()
            ))),
        );

        let first = identity.path("apply").unwrap().unwrap().to_path_buf();
        let second = identity.path("apply").unwrap().unwrap().to_path_buf();

        assert_eq!(first, second);
        assert_eq!(std::fs::read_to_string(&counter).unwrap(), "x");
    }

    #[test]
    fn dropping_the_identity_takes_the_provided_file_along() {
        let path = {
            let mut identity = Identity::new(
                None,
                Some(Provider::Line(
                    "printf 'AGE-SECRET-KEY-1TEST\n'".to_string(),
                )),
            );
            identity.path("apply").unwrap().unwrap().to_path_buf()
        };

        assert!(std::fs::symlink_metadata(&path).is_err());
    }

    #[test]
    fn a_failing_provider_reports_its_stderr() {
        let mut identity = Identity::new(
            None,
            Some(Provider::Line("echo locked >&2; exit 1".to_string())),
        );

        let err = identity.path("apply").unwrap_err().to_string();

        assert!(err.contains("apply: `echo locked >&2; exit 1` failed: locked"));
    }

    #[test]
    fn a_silent_provider_is_refused() {
        let mut identity = Identity::new(None, Some(Provider::Line("printf ' '".to_string())));

        let err = identity.path("apply").unwrap_err().to_string();

        assert!(err.contains("produced no identity"));
    }

    #[test]
    fn a_provider_that_cannot_run_says_so() {
        let mut identity = Identity::new(None, Some(program(&["luadot-no-such-provider"])));

        let err = format!("{:#}", identity.path("apply").unwrap_err());

        assert!(err.contains("apply: failed to run `luadot-no-such-provider`"));
    }
}
