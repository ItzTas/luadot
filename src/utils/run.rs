use std::path::Path;
use std::sync::{Arc, OnceLock};

use anyhow::Result;

use super::custom::{customized, said};
use super::workspace::configured;
use crate::backup::Backup;
use crate::files::SyncOutcome;
use crate::hook::Hooks;
use crate::lua::{Command, Config, Moment, Shared};
use crate::output;

static DRY_RUN: OnceLock<bool> = OnceLock::new();

static COMMAND: OnceLock<Command> = OnceLock::new();

static STARTED: OnceLock<Shared> = OnceLock::new();

pub fn set_dry_run(dry_run: bool) {
    let _ = DRY_RUN.set(dry_run);
}

pub fn dry_run() -> bool {
    DRY_RUN.get().copied().unwrap_or(false)
}

pub fn set_command(command: Command) {
    let _ = COMMAND.set(command);
}

pub fn started(shared: &Shared) -> Result<()> {
    let Some(command) = COMMAND.get().copied() else {
        return Ok(());
    };
    if STARTED.set(Arc::clone(shared)).is_err() {
        return Ok(());
    }

    fire(command, shared, Moment::Before)
}

pub fn finished() -> Result<()> {
    let (Some(command), Some(shared)) = (COMMAND.get().copied(), STARTED.get()) else {
        return Ok(());
    };

    fire(command, shared, Moment::After)
}

fn fire(command: Command, shared: &Shared, moment: Moment) -> Result<()> {
    let custom = configured(command.name(), shared)?
        .around(command)
        .and_then(|around| around.get(moment))
        .cloned();
    let Some(custom) = custom else {
        return Ok(());
    };

    said(custom.shown(
        &customized(command.name(), &command.call(), moment.key()),
        (),
    )?);

    Ok(())
}

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

    pub fn open(command: &str, dry_run: bool, config: &Config) -> Result<Self> {
        let backup = match dry_run || !config.backup() {
            true => None,
            false => Some(Backup::open(
                command,
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

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;
    use crate::lua::from_source;

    fn shared(source: &str) -> Shared {
        Arc::new(Mutex::new(from_source(source).unwrap()))
    }

    #[test]
    fn a_function_that_breaks_names_the_command_the_call_and_the_moment() {
        let shared = shared(r#"ld.on.tmpl.alt({ before = function() error("broken") end })"#);

        let err = format!(
            "{:#}",
            fire(Command::TmplAlt, &shared, Moment::Before).unwrap_err()
        );

        assert!(err.contains("tmpl alt: `ld.on.tmpl.alt`: `before` failed"));
        assert!(err.contains("broken"));
    }

    #[test]
    fn a_command_nothing_was_set_on_runs_nothing() {
        let shared = shared(r#"ld.on.apply({ after = function() error("wrong command") end })"#);

        fire(Command::Add, &shared, Moment::After).unwrap();
        fire(Command::Apply, &shared, Moment::Before).unwrap();
    }
}
