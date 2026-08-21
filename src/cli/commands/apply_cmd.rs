use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::Args;

use crate::crypt;
use crate::files::{self, ConflictPolicy, FileStatus, SyncOutcome};
use crate::lua::Config;
use crate::output;
use crate::utils::{self, Run, Workspace};

struct Secrets<'a> {
    config: &'a Config,
    ahead: crypt::Ahead,
}

struct Target<'a> {
    config: &'a Config,
    relative: &'a Path,
    dest: &'a Path,
    policy: ConflictPolicy,
}

impl<'a> Target<'a> {
    fn new(config: &'a Config, relative: &'a Path, dest: &'a Path) -> Self {
        Self {
            config,
            relative,
            dest,
            policy: config.conflict_policy(relative),
        }
    }

    fn settle(
        &self,
        run: &mut Run,
        status: Result<FileStatus>,
        sync: impl FnOnce() -> Result<SyncOutcome>,
    ) -> Result<SyncOutcome> {
        let dest = self.dest;
        let status =
            status.with_context(|| format!("apply: failed to inspect {}", dest.display()))?;
        let predicted = files::predict(self.policy, status, dest)
            .with_context(|| format!("apply: failed to apply {}", dest.display()))?;

        run.settle(
            predicted,
            self.relative,
            dest,
            self.config.on_change(self.relative),
            || sync().with_context(|| format!("apply: failed to apply {}", dest.display())),
        )
    }
}

#[derive(Debug, Args)]
pub struct ApplyArgs {
    #[arg(value_name = "PATH", help = "Narrow the run to this file or directory")]
    pub path: Option<String>,
    #[arg(
        short = 'n',
        long,
        help = "Report what would change, touching nothing and taking no backup"
    )]
    pub dry_run: bool,
}

pub fn apply_cmd(args: ApplyArgs) -> Result<()> {
    let Workspace { config, home, repo } = utils::workspace("apply")?;
    let config = utils::configured("apply", &config)?;

    let root = utils::managed_root("apply", &home, &repo, args.path.as_deref())?;

    let files = utils::managed_files("apply", &repo, &root, |relative| {
        config.is_ignored(&crypt::logical(relative))
    })?;
    if files.is_empty() {
        output::note("nothing to apply");
        return Ok(());
    }

    let lock = config.crypt_lock();
    let mut identity = config.crypt_identity(&home);
    require_plugins(&repo, &files, lock, &mut identity)?;

    let mut secrets = Secrets {
        config: &config,
        ahead: crypt::Ahead::new("apply", lock, identity, sources(&repo, &files)),
    };

    let mut run = Run::open("apply", args.dry_run, &config)?;

    let mut created = 0u32;
    let mut replaced = 0u32;
    let mut unchanged = 0u32;
    let mut skipped = 0u32;
    for file in &files {
        let relative = utils::relative(&repo, file);

        let outcome = match crypt::split(relative) {
            Some((stripped, backend)) => place_encrypted(
                &mut secrets,
                backend,
                &stripped,
                file,
                &home,
                &repo,
                &mut run,
            )?,
            None => {
                let dest = utils::system_path(&home, &repo, file)?;
                place_file(&config, relative, file, &dest, &mut run)?
            }
        };

        match outcome {
            SyncOutcome::Created => created += 1,
            SyncOutcome::Replaced => replaced += 1,
            SyncOutcome::AlreadySynced => unchanged += 1,
            SyncOutcome::Skipped => skipped += 1,
        }
    }

    output::note(format!(
        "{} {} file(s) ({created} created, {replaced} replaced, {unchanged} unchanged, {skipped} skipped)",
        match args.dry_run {
            true => "would apply",
            false => "applied",
        },
        files.len()
    ));
    run.finish("apply")?;

    Ok(())
}

fn sources(repo: &Path, files: &[PathBuf]) -> Vec<(crypt::Backend, PathBuf)> {
    files
        .iter()
        .filter_map(|file| {
            let (_, backend) = crypt::split(utils::relative(repo, file))?;
            Some((backend, file.clone()))
        })
        .collect()
}

fn require_plugins(
    repo: &Path,
    files: &[PathBuf],
    lock: crypt::Lock,
    identity: &mut crypt::Identity,
) -> Result<()> {
    let decrypts = files.iter().any(|file| {
        matches!(
            crypt::split(utils::relative(repo, file)),
            Some((_, crypt::Backend::Age))
        )
    });
    if !decrypts {
        return Ok(());
    }

    crypt::require_identity_plugins("apply", crypt::Backend::Age, lock, identity.path("apply")?)
}

fn place_file(
    config: &Config,
    relative: &Path,
    file: &Path,
    dest: &Path,
    run: &mut Run,
) -> Result<SyncOutcome> {
    let target = Target::new(config, relative, dest);
    let placement = config.placement(relative);

    target.settle(run, files::file_status(placement, file, dest), || {
        files::sync_file(target.policy, placement, file, dest)
    })
}

fn place_encrypted(
    secrets: &mut Secrets,
    backend: crypt::Backend,
    stripped: &Path,
    file: &Path,
    home: &Path,
    repo: &Path,
    run: &mut Run,
) -> Result<SyncOutcome> {
    let config = secrets.config;
    let dest = utils::system_path(home, repo, &repo.join(stripped))?;

    let contents = secrets
        .ahead
        .take(backend, file)
        .with_context(|| format!("apply: failed to decrypt {}", file.display()))?;

    let target = Target::new(config, stripped, &dest);
    let placement = config.placement(stripped);

    target.settle(
        run,
        crypt::plain_status("apply", &contents, &dest, placement.mode()),
        || crypt::place("apply", target.policy, placement, &contents, &dest),
    )
}
