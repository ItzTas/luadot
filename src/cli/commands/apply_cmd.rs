use std::path::Path;

use anyhow::{Context, Result, bail};
use clap::Args;

use crate::crypt;
use crate::files::{self, SyncOutcome};
use crate::lua::Config;
use crate::output;
use crate::utils::{self, Run, Workspace};

#[derive(Debug, Args)]
pub struct ApplyArgs {
    #[arg(value_name = "PATH")]
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

    let root = utils::managed_root("apply", &home, &repo, args.path.as_deref())?;

    let files = utils::managed_files("apply", &repo, &root, |relative| {
        config.is_ignored(&crypt::logical(relative))
    })?;
    if files.is_empty() {
        output::note("nothing to apply");
        return Ok(());
    }

    let mut run = Run::open("apply", args.dry_run, &home, &config)?;

    let mut created = 0u32;
    let mut replaced = 0u32;
    let mut unchanged = 0u32;
    let mut skipped = 0u32;
    for file in &files {
        let relative = utils::relative(&repo, file);

        let outcome = match crypt::split(relative) {
            Some((stripped, backend)) => {
                place_encrypted(&config, backend, &stripped, file, &home, &repo, &mut run)?
            }
            None => {
                let dest = utils::system_path(&home, &repo, file)?;
                match utils::is_root(relative) {
                    true => place_root(&config, relative, file, &dest, &mut run)?,
                    false => place_home(&config, relative, file, &dest, &mut run)?,
                }
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

fn place_home(
    config: &Config,
    relative: &Path,
    file: &Path,
    dest: &Path,
    run: &mut Run,
) -> Result<SyncOutcome> {
    let mode = config.link_mode(relative);
    let policy = config.conflict_policy(relative);

    let status = files::file_status(mode, file, dest)
        .with_context(|| format!("apply: failed to inspect {}", dest.display()))?;
    let predicted = files::predict(policy, status, dest)
        .with_context(|| format!("apply: failed to apply {}", dest.display()))?;

    run.settle(
        predicted,
        relative,
        dest,
        config.on_change(relative),
        || {
            files::sync_file(policy, mode, file, dest)
                .with_context(|| format!("apply: failed to apply {}", dest.display()))
        },
    )
}

fn place_encrypted(
    config: &Config,
    backend: crypt::Backend,
    stripped: &Path,
    file: &Path,
    home: &Path,
    repo: &Path,
    run: &mut Run,
) -> Result<SyncOutcome> {
    if utils::is_root(stripped) {
        bail!(
            "apply: {}: encrypted system files are not supported",
            stripped.display()
        );
    }

    let dest = utils::system_path(home, repo, &repo.join(stripped))?;
    let policy = config.conflict_policy(stripped);
    let identity = config
        .crypt_identity()
        .map(|path| utils::expand(home, path));

    let contents = crypt::decrypt("apply", backend, identity.as_deref(), file)
        .with_context(|| format!("apply: failed to decrypt {}", file.display()))?;
    let status = crypt::plain_status("apply", &contents, &dest)
        .with_context(|| format!("apply: failed to inspect {}", dest.display()))?;
    let predicted = files::predict(policy, status, &dest)
        .with_context(|| format!("apply: failed to apply {}", dest.display()))?;

    run.settle(
        predicted,
        stripped,
        &dest,
        config.on_change(stripped),
        || {
            crypt::place("apply", policy, &contents, &dest)
                .with_context(|| format!("apply: failed to apply {}", dest.display()))
        },
    )
}

fn place_root(
    config: &Config,
    relative: &Path,
    file: &Path,
    dest: &Path,
    run: &mut Run,
) -> Result<SyncOutcome> {
    let policy = config.conflict_policy(relative);
    let mode = config.mode(relative);

    let status = files::escalated_status(file, dest, mode)
        .with_context(|| format!("apply: failed to inspect {}", dest.display()))?;
    let predicted = files::predict(policy, status, dest)
        .with_context(|| format!("apply: failed to apply {}", dest.display()))?;

    run.settle(
        predicted,
        relative,
        dest,
        config.on_change(relative),
        || {
            files::sync_system(policy, file, dest, mode, config.owner(relative))
                .with_context(|| format!("apply: failed to apply {}", dest.display()))
        },
    )
}
