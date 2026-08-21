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
}
