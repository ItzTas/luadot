use std::path::Path;

use anyhow::Result;

use crate::backup::Backup;
use crate::files::SyncOutcome;
use crate::hook::Hooks;
use crate::lua::Config;
use crate::output;

#[derive(Debug, Default)]
pub struct Run {
    dry_run: bool,
    backup: Option<Backup>,
    hooks: Hooks,
}

impl Run {
    pub fn new(dry_run: bool, backup: Option<Backup>) -> Self {
        Self {
            dry_run,
            backup,
            hooks: Hooks::new(dry_run),
        }
    }

    pub fn open(command: &str, dry_run: bool, home: &Path, config: &Config) -> Result<Self> {
        let backup = match dry_run || !config.backup() {
            true => None,
            false => Some(Backup::open(
                command,
                home,
                config.backup_dir(),
                config.retention(),
            )?),
        };

        Ok(Self::new(dry_run, backup))
    }

    pub fn settle(
        &mut self,
        predicted: SyncOutcome,
        relative: &Path,
        dest: &Path,
        on_change: Option<&str>,
        sync: impl FnOnce() -> Result<SyncOutcome>,
    ) -> Result<SyncOutcome> {
        if self.dry_run {
            output::preview(predicted, relative.display());
            self.hooks.record(predicted, on_change);
            return Ok(predicted);
        }

        if predicted == SyncOutcome::Replaced
            && let Some(backup) = self.backup.as_mut()
        {
            backup.save(dest)?;
        }

        let outcome = sync()?;
        output::report(outcome, relative.display());
        self.hooks.record(outcome, on_change);

        Ok(outcome)
    }

    pub fn finish(&self, command: &str) -> Result<()> {
        if let Some(backup) = self.backup.as_ref() {
            backup.finish()?;
        }

        self.hooks.finish(command)
    }
}
