use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use super::paths::{relative, system_path};
use super::run::{Incoming, Run};
use super::units::{Managed, Unit, units, whole_link};
use super::workspace::{Workspace, configured, managed_files, managed_root, workspace};
use crate::crypt;
use crate::files::{self, ConflictPolicy, FileStatus, SyncOutcome};
use crate::lua::Config;
use crate::output;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Scope {
    Everything,
    Untouched,
}

impl Scope {
    fn covers(self, status: FileStatus) -> bool {
        match self {
            Self::Everything => true,
            Self::Untouched => status != FileStatus::Differs,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Placer {
    command: &'static str,
    scope: Scope,
    empty: &'static str,
    planned: &'static str,
    settled: &'static str,
}

impl Placer {
    pub const APPLY: Self = Self {
        command: "apply",
        scope: Scope::Everything,
        empty: "nothing to apply",
        planned: "would apply",
        settled: "applied",
    };

    pub const RELINK: Self = Self {
        command: "relink",
        scope: Scope::Untouched,
        empty: "nothing to relink",
        planned: "would relink",
        settled: "relinked",
    };

    pub fn place(&self, path: Option<&str>, dry_run: bool) -> Result<()> {
        let command = self.command;
        let Workspace { config, home, repo } = workspace(command)?;
        let config = configured(command, &config)?;

        let root = managed_root(command, &home, &repo, path)?;
        let found = managed_files(command, &repo, &root, |relative| {
            config.is_ignored(&crypt::logical(relative))
        })?;
        if found.is_empty() {
            output::note(self.empty);
            return Ok(());
        }
        let managed = units(command, &config, &repo, found)?;

        let mut run = Run::open(command, dry_run, &config)?;
        let placed = self.placed(&config, &home, &repo, &managed, &mut run)?;

        output::note(placed.line(match dry_run {
            true => self.planned,
            false => self.settled,
        }));

        run.finish(command)
    }

    fn placed(
        &self,
        config: &Config,
        home: &Path,
        repo: &Path,
        managed: &[Managed],
        run: &mut Run,
    ) -> Result<Placed> {
        let command = self.command;
        let files: Vec<PathBuf> = managed
            .iter()
            .filter_map(|one| match one {
                Managed::File(file) => Some(file.clone()),
                Managed::Unit(_) => None,
            })
            .collect();

        let lock = config.crypt_lock();
        let mut identity = config.crypt_identity(home);
        require_plugins(command, repo, &files, lock, &mut identity)?;

        let placing = Placing {
            command,
            scope: self.scope,
            config,
            home,
            repo,
        };
        let mut secrets = Secrets {
            ahead: crypt::Ahead::new(command, lock, identity, sources(repo, &files)),
        };

        let mut placed = Placed {
            total: managed.len(),
            ..Placed::default()
        };
        for one in managed {
            let outcome = match one {
                Managed::Unit(unit) => placing.unit(unit, run)?,
                Managed::File(file) => placing.file(&mut secrets, file, run)?,
            };

            placed.record(outcome);
        }

        Ok(placed)
    }
}

#[derive(Debug, Default)]
struct Placed {
    total: usize,
    created: u32,
    replaced: u32,
    unchanged: u32,
    skipped: u32,
}

impl Placed {
    fn line(&self, verb: &str) -> String {
        format!(
            "{verb} {} path(s) ({} created, {} replaced, {} unchanged, {} skipped)",
            self.total, self.created, self.replaced, self.unchanged, self.skipped
        )
    }

    fn record(&mut self, outcome: SyncOutcome) {
        let counted = match outcome {
            SyncOutcome::Created => &mut self.created,
            SyncOutcome::Replaced => &mut self.replaced,
            SyncOutcome::AlreadySynced => &mut self.unchanged,
            SyncOutcome::Skipped => &mut self.skipped,
        };

        *counted += 1;
    }
}

struct Placing<'a> {
    command: &'a str,
    scope: Scope,
    config: &'a Config,
    home: &'a Path,
    repo: &'a Path,
}

struct Target<'a> {
    placing: &'a Placing<'a>,
    relative: &'a Path,
    dest: &'a Path,
    policy: ConflictPolicy,
}

struct Secrets {
    ahead: crypt::Ahead,
}

impl<'a> Placing<'a> {
    fn target(&'a self, relative: &'a Path, dest: &'a Path) -> Target<'a> {
        Target {
            placing: self,
            relative,
            dest,
            policy: self.config.conflict_policy(relative),
        }
    }

    fn unit(&self, unit: &Unit, run: &mut Run) -> Result<SyncOutcome> {
        let relative = relative(self.repo, unit.root());
        let dest = system_path(self.home, self.repo, unit.root())?;
        let link = whole_link(self.command, self.config, relative)?;
        let target = self.target(relative, &dest);

        target.settle(
            run,
            files::dir_status(link, unit.root(), &dest),
            Incoming::Tree,
            || files::sync_dir(target.policy, link, unit.root(), &dest),
        )
    }

    fn file(&self, secrets: &mut Secrets, file: &Path, run: &mut Run) -> Result<SyncOutcome> {
        let relative = relative(self.repo, file);
        let Some((stripped, backend)) = crypt::split(relative) else {
            let dest = system_path(self.home, self.repo, file)?;
            return self.plain(relative, file, &dest, run);
        };

        self.encrypted(secrets, backend, &stripped, file, run)
    }

    fn plain(
        &self,
        relative: &Path,
        file: &Path,
        dest: &Path,
        run: &mut Run,
    ) -> Result<SyncOutcome> {
        let target = self.target(relative, dest);
        let placement = self.config.placement(relative);

        target.settle(
            run,
            files::file_status(placement, file, dest),
            Incoming::File(file),
            || files::sync_file(target.policy, placement, file, dest),
        )
    }

    fn encrypted(
        &self,
        secrets: &mut Secrets,
        backend: crypt::Backend,
        stripped: &Path,
        file: &Path,
        run: &mut Run,
    ) -> Result<SyncOutcome> {
        let command = self.command;
        let dest = system_path(self.home, self.repo, &self.repo.join(stripped))?;

        let contents = secrets
            .ahead
            .take(backend, file)
            .with_context(|| format!("{command}: failed to decrypt {}", file.display()))?;

        let target = self.target(stripped, &dest);
        let placement = self.config.placement(stripped);

        target.settle(
            run,
            crypt::plain_status(command, &contents, &dest, placement.mode()),
            Incoming::Bytes(&contents),
            || crypt::place(command, target.policy, placement, &contents, &dest),
        )
    }
}

impl Target<'_> {
    fn settle(
        &self,
        run: &mut Run,
        status: Result<FileStatus>,
        incoming: Incoming<'_>,
        sync: impl FnOnce() -> Result<SyncOutcome>,
    ) -> Result<SyncOutcome> {
        let command = self.placing.command;
        let dest = self.dest;
        let status =
            status.with_context(|| format!("{command}: failed to inspect {}", dest.display()))?;

        if !self.placing.scope.covers(status) {
            return Ok(run.left(self.relative));
        }

        let predicted = files::predict(self.policy, status, dest)
            .with_context(|| format!("{command}: failed to place {}", dest.display()))?;

        run.settle(
            predicted,
            self.relative,
            dest,
            self.placing.config.on_change(self.relative),
            incoming,
            || sync().with_context(|| format!("{command}: failed to place {}", dest.display())),
        )
    }
}

fn sources(repo: &Path, files: &[PathBuf]) -> Vec<(crypt::Backend, PathBuf)> {
    files
        .iter()
        .filter_map(|file| {
            let (_, backend) = crypt::split(relative(repo, file))?;
            Some((backend, file.clone()))
        })
        .collect()
}

fn require_plugins(
    command: &str,
    repo: &Path,
    files: &[PathBuf],
    lock: crypt::Lock,
    identity: &mut crypt::Identity,
) -> Result<()> {
    let decrypts = files.iter().any(|file| {
        matches!(
            crypt::split(relative(repo, file)),
            Some((_, crypt::Backend::Age))
        )
    });
    if !decrypts {
        return Ok(());
    }

    crypt::require_identity_plugins(command, crypt::Backend::Age, lock, identity.path(command)?)
}
