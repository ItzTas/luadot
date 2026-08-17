use std::process::{Command, ExitStatus};

use anyhow::{Context, Result, bail};
use tracing::debug;

use super::constants::{SHELL, SHELL_ARG};

pub(super) fn hook(command: &str, line: &str) -> Result<()> {
    debug!(line, "running");

    let status = Command::new(SHELL)
        .arg(SHELL_ARG)
        .arg(line)
        .status()
        .with_context(|| format!("{command}: failed to run `{line}`"))?;

    if !status.success() {
        bail!("{command}: `{line}` exited with status {}", code(status));
    }

    Ok(())
}

fn code(status: ExitStatus) -> String {
    status
        .code()
        .map_or_else(|| "signal".to_string(), |code| code.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runs_the_line_through_the_shell() {
        let dir = tempfile::tempdir().unwrap();
        let touched = dir.path().join("restarted");

        hook("alt", &format!("printf ok > {}", touched.display())).unwrap();

        assert_eq!(std::fs::read_to_string(&touched).unwrap(), "ok");
    }

    #[test]
    fn a_failing_line_reports_the_command_and_the_status() {
        let err = hook("alt", "exit 4").unwrap_err().to_string();

        assert_eq!(err, "alt: `exit 4` exited with status 4");
    }

    #[test]
    fn a_line_the_shell_cannot_run_is_reported() {
        let err = hook("alt", "luadot-no-such-program 2>/dev/null")
            .unwrap_err()
            .to_string();

        assert!(err.contains("exited with status 127"));
    }
}
