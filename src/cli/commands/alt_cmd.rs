use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use clap::Args;

use crate::files::{self, Entry, SyncOutcome};
use crate::lua::{self, Config, Content, Output};
use crate::output;
use crate::state::{self, Classes};
use crate::utils::{self, Backup};

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

#[derive(Debug, Default)]
struct Run {
    dry_run: bool,
    backup: Option<Backup>,
}

pub fn alt_cmd(args: AltArgs) -> Result<()> {
    let repo = utils::require_repo("alt")?;

    let home = utils::home_dir()?;
    let config = lua::load_config()?;
    let classes = state::load()?.classes().clone();

    let root = match args.path.as_deref() {
        Some(path) => template_root(&home, &repo, path)?,
        None => repo.clone(),
    };

    let templates: Vec<PathBuf> = files::collect_entries("alt", &root)?
        .into_iter()
        .filter(|entry| !config.is_ignored(utils::relative(&repo, &entry.target())))
        .filter_map(|entry| match entry {
            Entry::Template(dir) => Some(dir),
            Entry::File(_) => None,
        })
        .collect();
    if templates.is_empty() {
        output::note("no template to resolve");
        return Ok(());
    }

    let mut run = Run {
        dry_run: args.dry_run,
        backup: match args.dry_run || !config.backup() {
            true => None,
            false => Some(Backup::open("alt", &home)?),
        },
    };

    let mut outcomes: Vec<SyncOutcome> = Vec::new();
    for dir in &templates {
        outcomes.extend(resolve(&config, &home, &repo, dir, &classes, &mut run)?);
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
    if let Some(backup) = run.backup.as_ref() {
        backup.report();
    }

    Ok(())
}

fn template_root(home: &Path, repo: &Path, arg: &str) -> Result<PathBuf> {
    let target = std::path::absolute(arg).with_context(|| format!("alt: invalid path {arg}"))?;
    let managed = utils::repo_path(home, repo, &target)?;

    if let Some(dir) = files::template_dir(&managed).filter(|dir| dir.is_dir()) {
        return Ok(dir);
    }
    if managed.is_dir() {
        return Ok(managed);
    }

    bail!(
        "alt: {} has no template in the repository",
        target.display()
    )
}

fn resolve(
    config: &Config,
    home: &Path,
    repo: &Path,
    dir: &Path,
    classes: &Classes,
    run: &mut Run,
) -> Result<Vec<SyncOutcome>> {
    lua::load_template("alt", home, repo, dir, classes)?
        .iter()
        .map(|output| place(config, home, output, run))
        .collect()
}

fn place(config: &Config, home: &Path, output: &Output, run: &mut Run) -> Result<SyncOutcome> {
    let relative = utils::relative(home, output.dest());
    let policy = output
        .conflict()
        .unwrap_or_else(|| config.conflict_policy(relative));
    let mode = output.link().unwrap_or_else(|| config.link_mode(relative));

    let status = match output.content() {
        Content::File(source) => files::file_status(mode, source, output.dest()),
        Content::Text(text) => files::text_status(output.dest(), text),
    }
    .with_context(|| format!("alt: failed to inspect {}", output.dest().display()))?;

    let predicted = files::predict(policy, status, output.dest())
        .with_context(|| format!("alt: failed to place {}", output.dest().display()))?;

    if run.dry_run {
        utils::preview(predicted, relative.display());
        return Ok(predicted);
    }

    if predicted == SyncOutcome::Replaced
        && let Some(backup) = run.backup.as_mut()
    {
        backup.save(output.dest())?;
    }

    match output.content() {
        Content::File(source) => files::sync_file(policy, mode, source, output.dest()),
        Content::Text(text) => files::write_file(policy, output.dest(), text),
    }
    .with_context(|| format!("alt: failed to place {}", output.dest().display()))
}

fn count(outcomes: &[SyncOutcome], kind: SyncOutcome) -> usize {
    outcomes.iter().filter(|outcome| **outcome == kind).count()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(path: &Path, contents: &str) {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(path, contents).unwrap();
    }

    #[test]
    fn counts_group_the_outcomes() {
        let outcomes = [
            SyncOutcome::Created,
            SyncOutcome::Created,
            SyncOutcome::Skipped,
        ];

        assert_eq!(count(&outcomes, SyncOutcome::Created), 2);
        assert_eq!(count(&outcomes, SyncOutcome::Skipped), 1);
        assert_eq!(count(&outcomes, SyncOutcome::Replaced), 0);
    }

    #[test]
    fn a_selected_variant_lands_on_the_mirrored_path() {
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
            &Config::default(),
            &home,
            &repo,
            &dir,
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
    fn generated_content_is_written_and_then_left_alone() {
        let root = tempfile::tempdir().unwrap();
        let home = root.path().join("home");
        let repo = root.path().join("repo");
        let dir = repo.join(".config/nvim/init.lua.luadot");
        write(
            &dir.join("luadot.lua"),
            r#"return "vim.g.mapleader = ' '\n""#,
        );

        let outcomes = resolve(
            &Config::default(),
            &home,
            &repo,
            &dir,
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
            &Config::default(),
            &home,
            &repo,
            &dir,
            &Classes::default(),
            &mut Run::default(),
        )
        .unwrap();

        assert_eq!(outcomes, vec![SyncOutcome::AlreadySynced]);
    }

    #[test]
    fn a_template_overrides_the_configured_link_mode() {
        let root = tempfile::tempdir().unwrap();
        let home = root.path().join("home");
        let repo = root.path().join("repo");
        let dir = repo.join(".zshrc.luadot");
        write(&dir.join("laptop.zsh"), "laptop");
        write(
            &dir.join("luadot.lua"),
            r#"return { content = ld.alt.file("laptop.zsh"), link = "symbolic" }"#,
        );

        resolve(
            &Config::default(),
            &home,
            &repo,
            &dir,
            &Classes::default(),
            &mut Run::default(),
        )
        .unwrap();

        assert_eq!(
            std::fs::read_link(home.join(".zshrc")).unwrap(),
            dir.join("laptop.zsh")
        );
    }

    #[test]
    fn a_declared_destination_leaves_the_mirrored_path_alone() {
        let root = tempfile::tempdir().unwrap();
        let home = root.path().join("home");
        let repo = root.path().join("repo");
        let dir = repo.join(".zshrc.luadot");
        write(
            &dir.join("luadot.lua"),
            r#"return { dest = "~/.config/zsh/.zshrc", content = "generated\n" }"#,
        );

        resolve(
            &Config::default(),
            &home,
            &repo,
            &dir,
            &Classes::default(),
            &mut Run::default(),
        )
        .unwrap();

        assert!(!home.join(".zshrc").exists());
        assert_eq!(
            std::fs::read_to_string(home.join(".config/zsh/.zshrc")).unwrap(),
            "generated\n"
        );
    }

    #[test]
    fn the_configuration_still_drives_a_generated_file() {
        let root = tempfile::tempdir().unwrap();
        let home = root.path().join("home");
        let repo = root.path().join("repo");
        let dir = repo.join(".zshrc.luadot");
        write(&dir.join("luadot.lua"), r#"return "generated\n""#);
        write(&home.join(".zshrc"), "handwritten\n");

        let config = lua::from_source(r#"ld.git.conflict("skip")"#).unwrap();
        let outcomes = resolve(
            &config,
            &home,
            &repo,
            &dir,
            &Classes::default(),
            &mut Run::default(),
        )
        .unwrap();

        assert_eq!(outcomes, vec![SyncOutcome::Skipped]);
        assert_eq!(
            std::fs::read_to_string(home.join(".zshrc")).unwrap(),
            "handwritten\n"
        );
    }

    #[test]
    fn a_dry_run_reports_what_it_would_place_and_touches_nothing() {
        let root = tempfile::tempdir().unwrap();
        let home = root.path().join("home");
        let repo = root.path().join("repo");
        let dir = repo.join(".zshrc.luadot");
        write(&dir.join("luadot.lua"), r#"return "generated\n""#);

        let mut run = Run {
            dry_run: true,
            backup: None,
        };
        let outcomes = resolve(
            &Config::default(),
            &home,
            &repo,
            &dir,
            &Classes::default(),
            &mut run,
        )
        .unwrap();

        assert_eq!(outcomes, vec![SyncOutcome::Created]);
        assert!(!home.join(".zshrc").exists());
    }

    #[test]
    fn a_dry_run_over_a_diverging_file_reports_the_replacement() {
        let root = tempfile::tempdir().unwrap();
        let home = root.path().join("home");
        let repo = root.path().join("repo");
        let dir = repo.join(".zshrc.luadot");
        write(&dir.join("luadot.lua"), r#"return "generated\n""#);
        write(&home.join(".zshrc"), "handwritten\n");

        let mut run = Run {
            dry_run: true,
            backup: None,
        };
        let outcomes = resolve(
            &Config::default(),
            &home,
            &repo,
            &dir,
            &Classes::default(),
            &mut run,
        )
        .unwrap();

        assert_eq!(outcomes, vec![SyncOutcome::Replaced]);
        assert_eq!(
            std::fs::read_to_string(home.join(".zshrc")).unwrap(),
            "handwritten\n"
        );
    }

    #[test]
    fn a_replaced_file_is_backed_up_before_it_goes() {
        let root = tempfile::tempdir().unwrap();
        let home = root.path().join("home");
        let repo = root.path().join("repo");
        let dir = repo.join(".zshrc.luadot");
        write(&dir.join("luadot.lua"), r#"return "generated\n""#);
        write(&home.join(".zshrc"), "handwritten\n");

        let saved = root.path().join("backup");
        let mut run = Run {
            dry_run: false,
            backup: Some(Backup::at("alt", &home, saved.clone())),
        };
        resolve(
            &Config::default(),
            &home,
            &repo,
            &dir,
            &Classes::default(),
            &mut run,
        )
        .unwrap();

        assert_eq!(
            std::fs::read_to_string(home.join(".zshrc")).unwrap(),
            "generated\n"
        );
        assert_eq!(
            std::fs::read_to_string(saved.join(".zshrc")).unwrap(),
            "handwritten\n"
        );
    }

    #[test]
    fn a_template_is_reached_through_the_path_it_produces() {
        let root = tempfile::tempdir().unwrap();
        let home = root.path().join("home");
        let repo = root.path().join("repo");
        let dir = repo.join(".zshrc.luadot");
        write(&dir.join("luadot.lua"), r#"return "generated\n""#);

        let arg = home.join(".zshrc").to_string_lossy().into_owned();

        assert_eq!(template_root(&home, &repo, &arg).unwrap(), dir);
    }

    #[test]
    fn a_path_without_a_template_is_reported() {
        let root = tempfile::tempdir().unwrap();
        let home = root.path().join("home");
        let repo = root.path().join("repo");
        std::fs::create_dir_all(&repo).unwrap();

        let arg = home.join(".zshrc").to_string_lossy().into_owned();
        let err = template_root(&home, &repo, &arg).unwrap_err().to_string();

        assert!(err.contains("alt: "));
        assert!(err.contains("has no template in the repository"));
    }
}
