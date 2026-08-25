use anyhow::Result;

use super::constants::HOOK_LABEL;
use super::run::hook;
use crate::files::SyncOutcome;
use crate::output::{self, Tone};

#[derive(Debug, Default)]
pub struct Hooks {
    lines: Vec<String>,
    dry_run: bool,
}

impl Hooks {
    pub fn new(dry_run: bool) -> Self {
        Self {
            lines: Vec::new(),
            dry_run,
        }
    }

    pub fn record(&mut self, outcome: SyncOutcome, line: Option<&str>) {
        let Some(line) = line else {
            return;
        };
        if !matches!(outcome, SyncOutcome::Created | SyncOutcome::Replaced) {
            return;
        }
        if self.lines.iter().any(|current| current == line) {
            return;
        }

        self.lines.push(line.to_string());
    }

    pub fn finish(&self, command: &str) -> Result<()> {
        for line in &self.lines {
            if self.dry_run {
                output::entry(Tone::Muted, HOOK_LABEL, line);
                continue;
            }

            hook(command, line)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_a_changed_file_records() {
        let mut hooks = Hooks::default();
        hooks.record(SyncOutcome::Created, Some("created"));
        hooks.record(SyncOutcome::Replaced, Some("replaced"));
        hooks.record(SyncOutcome::AlreadySynced, Some("unchanged"));
        hooks.record(SyncOutcome::Skipped, Some("skipped"));
        hooks.record(SyncOutcome::Created, None);

        assert_eq!(hooks.lines, ["created", "replaced"]);
    }

    #[test]
    fn every_command_runs_once_at_the_end() {
        let dir = tempfile::tempdir().unwrap();
        let first = dir.path().join("first");
        let second = dir.path().join("second");

        let mut hooks = Hooks::new(false);
        hooks.record(SyncOutcome::Created, Some(&count(&first)));
        hooks.record(SyncOutcome::Created, Some(&count(&first)));
        hooks.record(SyncOutcome::Replaced, Some(&count(&second)));

        hooks.finish("alt").unwrap();

        assert_eq!(std::fs::read_to_string(&first).unwrap(), "x");
        assert_eq!(std::fs::read_to_string(&second).unwrap(), "x");
    }

    fn count(path: &std::path::Path) -> String {
        format!("printf x >> {}", path.display())
    }
}
