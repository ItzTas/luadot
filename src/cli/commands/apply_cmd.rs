use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::Args;

use crate::crypt;
use crate::files::{self, SyncOutcome};
use crate::lua::Config;
use crate::output;
use crate::utils::{self, Run, Workspace};

struct Secrets<'a> {
    config: &'a Config,
    lock: crypt::Lock,
    identity: crypt::Identity,
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

    let mut secrets = Secrets {
        config: &config,
        lock: config.crypt_lock(),
        identity: config.crypt_identity(&home),
    };
    require_plugins(&repo, &files, &mut secrets)?;

    let mut run = Run::open("apply", args.dry_run, &home, &config)?;

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

fn require_plugins(repo: &Path, files: &[PathBuf], secrets: &mut Secrets) -> Result<()> {
    let decrypts = files.iter().any(|file| {
        matches!(
            crypt::split(utils::relative(repo, file)),
            Some((_, crypt::Backend::Age))
        )
    });
    if !decrypts {
        return Ok(());
    }

    crypt::require_identity_plugins(
        "apply",
        crypt::Backend::Age,
        secrets.lock,
        secrets.identity.path("apply")?,
    )
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
    let policy = config.conflict_policy(stripped);

    let contents = crypt::decrypt(
        "apply",
        backend,
        secrets.lock,
        secrets.identity.path("apply")?,
        file,
    )
    .with_context(|| format!("apply: failed to decrypt {}", file.display()))?;

    if utils::is_root(stripped) {
        return place_root_secret(config, stripped, &contents, &dest, run);
    }

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

fn place_root_secret(
    config: &Config,
    relative: &Path,
    contents: &[u8],
    dest: &Path,
    run: &mut Run,
) -> Result<SyncOutcome> {
    let policy = config.conflict_policy(relative);
    let mode = config.mode(relative);

    let status = crypt::escalated_status("apply", contents, dest, mode)
        .with_context(|| format!("apply: failed to inspect {}", dest.display()))?;
    let predicted = files::predict(policy, status, dest)
        .with_context(|| format!("apply: failed to apply {}", dest.display()))?;

    run.settle(
        predicted,
        relative,
        dest,
        config.on_change(relative),
        || {
            crypt::place_system(
                "apply",
                policy,
                contents,
                dest,
                mode,
                config.owner(relative),
            )
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
