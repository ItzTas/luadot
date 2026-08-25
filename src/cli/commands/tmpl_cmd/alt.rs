use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use clap::Args;

use crate::files::{self, ConflictPolicy, Entry, SyncOutcome};
use crate::lua::{Config, Content, Output, Shared};
use crate::output;
use crate::state::{self, Classes};
use crate::utils::{self, Run, Workspace};

#[derive(Debug, Args)]
pub struct AltArgs {
    #[arg(value_name = "PATH")]
    pub path: Option<String>,
    #[arg(
        short = 'n',
        long,
        help = "Report what would change, touching nothing and taking no backup"
    )]
    pub dry_run: bool,
}

pub fn alt(args: AltArgs) -> Result<()> {
    let Workspace {
        config: shared,
        home,
        repo,
    } = utils::workspace("tmpl alt")?;
    let config = utils::configured("tmpl alt", &shared)?;

    let classes = state::load()?.classes().clone();

    let root = match args.path.as_deref() {
        Some(path) => template_root(&home, &repo, path)?,
        None => repo.clone(),
    };

    let templates: Vec<Entry> = utils::managed_entries("tmpl alt", &repo, &root, |relative| {
        config.is_ignored(relative)
    })?
    .into_iter()
    .filter(|entry| match entry {
        Entry::Template(_) | Entry::Standalone(_) => true,
        Entry::File(_) => false,
    })
    .collect();
    if templates.is_empty() {
        output::note("no template to resolve");
        return Ok(());
    }

    let mut run = Run::open("tmpl alt", args.dry_run, &config)?;
    drop(config);

    let mut outcomes: Vec<SyncOutcome> = Vec::new();
    for entry in &templates {
        outcomes.extend(resolve(&shared, &home, &repo, entry, &classes, &mut run)?);
    }

    output::note(format!(
        "{} {} template(s) into {} file(s) ({} created, {} replaced, {} unchanged, {} skipped)",
        match args.dry_run {
            true => "would resolve",
            false => "resolved",
        },
        templates.len(),
        outcomes.len(),
        count(&outcomes, SyncOutcome::Created),
        count(&outcomes, SyncOutcome::Replaced),
        count(&outcomes, SyncOutcome::AlreadySynced),
        count(&outcomes, SyncOutcome::Skipped),
    ));
    run.finish("tmpl alt")?;

    Ok(())
}

fn template_root(home: &Path, repo: &Path, arg: &str) -> Result<PathBuf> {
    let target =
        std::path::absolute(arg).with_context(|| format!("tmpl alt: invalid path {arg}"))?;
    let managed = utils::repo_path(home, repo, &target)
        .with_context(|| format!("tmpl alt: cannot resolve {}", target.display()))?;

    if let Some(dir) = files::template_dir(&managed).filter(|dir| dir.exists()) {
        return Ok(dir);
    }
    if managed.is_dir() {
        return Ok(managed);
    }

    bail!(
        "tmpl alt: {} has no template in the repository",
        target.display()
    )
}

fn resolve(
    shared: &Shared,
    home: &Path,
    repo: &Path,
    entry: &Entry,
    classes: &Classes,
    run: &mut Run,
) -> Result<Vec<SyncOutcome>> {
    let outputs = utils::outputs("tmpl alt", home, repo, entry, classes, shared)?;
    let config = utils::configured("tmpl alt", shared)?;

    outputs
        .iter()
        .map(|output| place(&config, home, output, run))
        .collect()
}

fn place(config: &Config, home: &Path, output: &Output, run: &mut Run) -> Result<SyncOutcome> {
    let relative = utils::output_relative("tmpl alt", home, output)?;
    let status = utils::output_status("tmpl alt", config, home, output)?;

    let policy = output
        .conflict()
        .unwrap_or_else(|| config.conflict_policy(&relative));
    let predicted = files::predict(policy, status, output.dest())
        .with_context(|| format!("tmpl alt: failed to place {}", output.dest().display()))?;
    let on_change = output.on_change().or_else(|| config.on_change(&relative));

    run.settle(predicted, &relative, output.dest(), on_change, || {
        write(config, &relative, policy, output)
    })
}

fn write(
    config: &Config,
    relative: &Path,
    policy: ConflictPolicy,
    output: &Output,
) -> Result<SyncOutcome> {
    let dest = output.dest();
    let placement = utils::output_placement(config, relative, output);

    match output.content() {
        Content::File(source) => files::sync_file(policy, placement, source, dest),
        Content::Text(text) => files::write_file(policy, placement.attributes(), dest, text),
    }
    .with_context(|| format!("tmpl alt: failed to place {}", dest.display()))
}

fn count(outcomes: &[SyncOutcome], kind: SyncOutcome) -> usize {
    outcomes.iter().filter(|outcome| **outcome == kind).count()
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    fn configuration() -> crate::lua::Shared {
        Arc::new(Mutex::new(Config::default()))
    }

    use super::*;
    use crate::backup::Backup;

    fn write(path: &Path, contents: &str) {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(path, contents).unwrap();
    }

    #[test]
    fn a_variant_lands_on_the_mirror() {
        let root = tempfile::tempdir().unwrap();
        let home = root.path().join("home");
        let repo = root.path().join("repo");
        let dir = repo.join(".zshrc.luadot");
        write(&dir.join("laptop.zsh"), "laptop");
        write(
            &dir.join("luadot.lua"),
            r#"return ld.alt.file("laptop.zsh")"#,
        );

        let outcomes = resolve(
            &configuration(),
            &home,
            &repo,
            &Entry::Template(dir.clone()),
            &Classes::default(),
            &mut Run::default(),
        )
        .unwrap();

        assert_eq!(outcomes, vec![SyncOutcome::Created]);
        assert_eq!(
            std::fs::read_to_string(home.join(".zshrc")).unwrap(),
            "laptop"
        );
    }

    #[test]
    fn generated_content_is_written_once() {
        let root = tempfile::tempdir().unwrap();
        let home = root.path().join("home");
        let repo = root.path().join("repo");
        let dir = repo.join(".config/nvim/init.lua.luadot");
        write(
            &dir.join("luadot.lua"),
            r#"return "vim.g.mapleader = ' '\n""#,
        );

        let outcomes = resolve(
            &configuration(),
            &home,
            &repo,
            &Entry::Template(dir.clone()),
            &Classes::default(),
            &mut Run::default(),
        )
        .unwrap();

        assert_eq!(outcomes, vec![SyncOutcome::Created]);
        assert_eq!(
            std::fs::read_to_string(home.join(".config/nvim/init.lua")).unwrap(),
            "vim.g.mapleader = ' '\n"
        );

        let outcomes = resolve(
            &configuration(),
            &home,
            &repo,
            &Entry::Template(dir.clone()),
            &Classes::default(),
            &mut Run::default(),
        )
        .unwrap();

        assert_eq!(outcomes, vec![SyncOutcome::AlreadySynced]);
    }

    #[test]
    fn a_replaced_file_is_backed_up() {
        let root = tempfile::tempdir().unwrap();
        let home = root.path().join("home");
        let repo = root.path().join("repo");
        let dir = repo.join(".zshrc.luadot");
        write(&dir.join("luadot.lua"), r#"return "generated\n""#);
        write(&home.join(".zshrc"), "handwritten\n");

        let saved = root.path().join("backup");
        let mut run = Run::new(false, Some(Backup::at("tmpl alt", saved.clone())));
        resolve(
            &configuration(),
            &home,
            &repo,
            &Entry::Template(dir.clone()),
            &Classes::default(),
            &mut run,
        )
        .unwrap();

        assert_eq!(
            std::fs::read_to_string(home.join(".zshrc")).unwrap(),
            "generated\n"
        );
        assert_eq!(
            std::fs::read_to_string(saved.join(home.strip_prefix("/").unwrap()).join(".zshrc"))
                .unwrap(),
            "handwritten\n"
        );
    }

    fn mode(path: &Path) -> u32 {
        use std::os::unix::fs::PermissionsExt;

        std::fs::metadata(path).unwrap().permissions().mode() & 0o7777
    }

    fn resolved(home: &Path, repo: &Path, dir: &Path, run: &mut Run) -> Vec<SyncOutcome> {
        resolve(
            &configuration(),
            home,
            repo,
            &Entry::Template(dir.to_path_buf()),
            &Classes::default(),
            run,
        )
        .unwrap()
    }

    #[test]
    fn a_declared_mode_lands_on_it() {
        let root = tempfile::tempdir().unwrap();
        let home = root.path().join("home");
        let repo = root.path().join("repo");
        let dir = repo.join(".netrc.luadot");
        write(
            &dir.join("luadot.lua"),
            r#"return { content = "machine example\n", mode = "600" }"#,
        );

        let outcomes = resolved(&home, &repo, &dir, &mut Run::default());

        assert_eq!(outcomes, vec![SyncOutcome::Created]);
        assert_eq!(mode(&home.join(".netrc")), 0o600);
    }

    fn resolved_with(
        config: &Config,
        home: &Path,
        repo: &Path,
        dir: &Path,
        run: &mut Run,
    ) -> Vec<SyncOutcome> {
        let outcomes = resolve(
            &Arc::new(Mutex::new(config.clone())),
            home,
            repo,
            &Entry::Template(dir.to_path_buf()),
            &Classes::default(),
            run,
        )
        .unwrap();
        run.finish("tmpl alt").unwrap();

        outcomes
    }

    #[test]
    fn a_command_runs_only_on_change() {
        let root = tempfile::tempdir().unwrap();
        let home = root.path().join("home");
        let repo = root.path().join("repo");
        let dir = repo.join(".config/mako/config.luadot");
        let restarted = root.path().join("restarted");
        write(
            &dir.join("luadot.lua"),
            &format!(
                r#"return {{ content = "font=monospace\n", on_change = "printf ok > {}" }}"#,
                restarted.display()
            ),
        );

        let config = Config::default();
        let outcomes = resolved_with(&config, &home, &repo, &dir, &mut Run::default());

        assert_eq!(outcomes, vec![SyncOutcome::Created]);
        assert_eq!(std::fs::read_to_string(&restarted).unwrap(), "ok");

        std::fs::remove_file(&restarted).unwrap();
        let outcomes = resolved_with(&config, &home, &repo, &dir, &mut Run::default());

        assert_eq!(outcomes, vec![SyncOutcome::AlreadySynced]);
        assert!(!restarted.exists());
    }
}
