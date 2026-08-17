use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use clap::Args;

use crate::files::{self, Entry, SyncOutcome};
use crate::lua::{self, Config, Content, Output};
use crate::output;
use crate::state::{self, Classes};
use crate::utils::{self, Run, Workspace};

use super::super::constants::SYSTEM_TEXT_MODE;

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

pub fn alt_cmd(args: AltArgs) -> Result<()> {
    let Workspace { config, home, repo } = utils::workspace("alt")?;

    let classes = state::load()?.classes().clone();

    let root = match args.path.as_deref() {
        Some(path) => template_root(&home, &repo, path)?,
        None => repo.clone(),
    };

    let templates: Vec<Entry> =
        utils::managed_entries("alt", &repo, &root, |relative| config.is_ignored(relative))?
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

    let mut run = Run::open("alt", args.dry_run, &home, &config)?;

    let mut outcomes: Vec<SyncOutcome> = Vec::new();
    for entry in &templates {
        outcomes.extend(resolve(&config, &home, &repo, entry, &classes, &mut run)?);
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
    run.finish("alt")?;

    Ok(())
}

fn template_root(home: &Path, repo: &Path, arg: &str) -> Result<PathBuf> {
    let target = std::path::absolute(arg).with_context(|| format!("alt: invalid path {arg}"))?;
    let managed = utils::repo_path(home, repo, &target)?;

    if let Some(dir) = files::template_dir(&managed).filter(|dir| dir.exists()) {
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
    entry: &Entry,
    classes: &Classes,
    run: &mut Run,
) -> Result<Vec<SyncOutcome>> {
    outputs(home, repo, entry, classes)?
        .iter()
        .map(|output| place(config, home, output, run))
        .collect()
}

fn outputs(home: &Path, repo: &Path, entry: &Entry, classes: &Classes) -> Result<Vec<Output>> {
    match entry {
        Entry::Template(dir) => lua::load_template("alt", home, repo, dir, classes),
        Entry::Standalone(path) => Ok(vec![lua::load_template_file(
            "alt", home, repo, path, classes,
        )?]),
        Entry::File(path) => bail!("alt: {} is not a template", path.display()),
    }
}

fn place(config: &Config, home: &Path, output: &Output, run: &mut Run) -> Result<SyncOutcome> {
    let relative = utils::managed_relative(home, output.dest())
        .with_context(|| format!("alt: failed to place {}", output.dest().display()))?;
    if utils::is_root(&relative) {
        return place_root(config, &relative, output, run);
    }

    let relative = relative.as_path();
    let policy = output
        .conflict()
        .unwrap_or_else(|| config.conflict_policy(relative));
    let mode = output.link().unwrap_or_else(|| config.link_mode(relative));

    let status = match output.content() {
        Content::File(source) => files::file_status(mode, source, output.dest()),
        Content::Text(text) => files::text_status(output.dest(), text, output.mode()),
    }
    .with_context(|| format!("alt: failed to inspect {}", output.dest().display()))?;

    let predicted = files::predict(policy, status, output.dest())
        .with_context(|| format!("alt: failed to place {}", output.dest().display()))?;

    let on_change = output.on_change().or_else(|| config.on_change(relative));

    run.settle(predicted, relative, output.dest(), on_change, || {
        match output.content() {
            Content::File(source) => files::sync_file(policy, mode, source, output.dest()),
            Content::Text(text) => files::write_file(policy, output.dest(), text, output.mode()),
        }
        .with_context(|| format!("alt: failed to place {}", output.dest().display()))
    })
}

fn place_root(
    config: &Config,
    relative: &Path,
    output: &Output,
    run: &mut Run,
) -> Result<SyncOutcome> {
    let dest = output.dest();
    let policy = output
        .conflict()
        .unwrap_or_else(|| config.conflict_policy(relative));

    let staged;
    let (source, mode) = match output.content() {
        Content::File(source) => (source.as_path(), config.mode(relative)),
        Content::Text(text) => {
            staged = files::stage_text(text)
                .with_context(|| format!("alt: failed to place {}", dest.display()))?;
            let mode = output
                .mode()
                .or_else(|| config.mode(relative))
                .unwrap_or(SYSTEM_TEXT_MODE);
            (staged.path(), Some(mode))
        }
    };

    let status = files::escalated_status(source, dest, mode)
        .with_context(|| format!("alt: failed to inspect {}", dest.display()))?;
    let predicted = files::predict(policy, status, dest)
        .with_context(|| format!("alt: failed to place {}", dest.display()))?;

    let on_change = output.on_change().or_else(|| config.on_change(relative));

    run.settle(predicted, relative, dest, on_change, || {
        files::sync_system(policy, source, dest, mode, config.owner(relative))
            .with_context(|| format!("alt: failed to place {}", dest.display()))
    })
}

fn count(outcomes: &[SyncOutcome], kind: SyncOutcome) -> usize {
    outcomes.iter().filter(|outcome| **outcome == kind).count()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backup::Backup;

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
        let dir = repo.join("home/.zshrc.luadot");
        write(&dir.join("laptop.zsh"), "laptop");
        write(
            &dir.join("luadot.lua"),
            r#"return ld.alt.file("laptop.zsh")"#,
        );

        let outcomes = resolve(
            &Config::default(),
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
    fn generated_content_is_written_and_then_left_alone() {
        let root = tempfile::tempdir().unwrap();
        let home = root.path().join("home");
        let repo = root.path().join("repo");
        let dir = repo.join("home/.config/nvim/init.lua.luadot");
        write(
            &dir.join("luadot.lua"),
            r#"return "vim.g.mapleader = ' '\n""#,
        );

        let outcomes = resolve(
            &Config::default(),
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
            &Config::default(),
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
    fn a_template_overrides_the_configured_link_mode() {
        let root = tempfile::tempdir().unwrap();
        let home = root.path().join("home");
        let repo = root.path().join("repo");
        let dir = repo.join("home/.zshrc.luadot");
        write(&dir.join("laptop.zsh"), "laptop");
        write(
            &dir.join("luadot.lua"),
            r#"return { content = ld.alt.file("laptop.zsh"), link = "symbolic" }"#,
        );

        resolve(
            &Config::default(),
            &home,
            &repo,
            &Entry::Template(dir.clone()),
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
        let dir = repo.join("home/.zshrc.luadot");
        write(
            &dir.join("luadot.lua"),
            r#"return { dest = "~/.config/zsh/.zshrc", content = "generated\n" }"#,
        );

        resolve(
            &Config::default(),
            &home,
            &repo,
            &Entry::Template(dir.clone()),
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
        let dir = repo.join("home/.zshrc.luadot");
        write(&dir.join("luadot.lua"), r#"return "generated\n""#);
        write(&home.join(".zshrc"), "handwritten\n");

        let config = lua::from_source(r#"ld.opt.conflict("skip")"#).unwrap();
        let outcomes = resolve(
            &config,
            &home,
            &repo,
            &Entry::Template(dir.clone()),
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
        let dir = repo.join("home/.zshrc.luadot");
        write(&dir.join("luadot.lua"), r#"return "generated\n""#);

        let mut run = Run::new(true, None);
        let outcomes = resolve(
            &Config::default(),
            &home,
            &repo,
            &Entry::Template(dir.clone()),
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
        let dir = repo.join("home/.zshrc.luadot");
        write(&dir.join("luadot.lua"), r#"return "generated\n""#);
        write(&home.join(".zshrc"), "handwritten\n");

        let mut run = Run::new(true, None);
        let outcomes = resolve(
            &Config::default(),
            &home,
            &repo,
            &Entry::Template(dir.clone()),
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
        let dir = repo.join("home/.zshrc.luadot");
        write(&dir.join("luadot.lua"), r#"return "generated\n""#);
        write(&home.join(".zshrc"), "handwritten\n");

        let saved = root.path().join("backup");
        let mut run = Run::new(false, Some(Backup::at("alt", &home, saved.clone())));
        resolve(
            &Config::default(),
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
            std::fs::read_to_string(saved.join("home/.zshrc")).unwrap(),
            "handwritten\n"
        );
    }

    #[test]
    fn a_template_is_reached_through_the_path_it_produces() {
        let root = tempfile::tempdir().unwrap();
        let home = root.path().join("home");
        let repo = root.path().join("repo");
        let dir = repo.join("home/.zshrc.luadot");
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

    #[test]
    fn a_standalone_template_lands_on_the_mirrored_path() {
        let root = tempfile::tempdir().unwrap();
        let home = root.path().join("home");
        let repo = root.path().join("repo");
        let file = repo.join("home/.zprofile.luadot");
        write(&file, "export HOST=<%= 1 + 1 %>\n");

        let outcomes = resolve(
            &Config::default(),
            &home,
            &repo,
            &Entry::Standalone(file),
            &Classes::default(),
            &mut Run::default(),
        )
        .unwrap();

        assert_eq!(outcomes, vec![SyncOutcome::Created]);
        assert_eq!(
            std::fs::read_to_string(home.join(".zprofile")).unwrap(),
            "export HOST=2\n"
        );
    }

    #[test]
    fn a_standalone_template_follows_the_configured_conflict_policy() {
        let root = tempfile::tempdir().unwrap();
        let home = root.path().join("home");
        let repo = root.path().join("repo");
        let file = repo.join("home/.zprofile.luadot");
        write(&file, "generated\n");
        write(&home.join(".zprofile"), "handwritten\n");

        let config = lua::from_source(r#"ld.opt.conflict("skip")"#).unwrap();
        let outcomes = resolve(
            &config,
            &home,
            &repo,
            &Entry::Standalone(file),
            &Classes::default(),
            &mut Run::default(),
        )
        .unwrap();

        assert_eq!(outcomes, vec![SyncOutcome::Skipped]);
        assert_eq!(
            std::fs::read_to_string(home.join(".zprofile")).unwrap(),
            "handwritten\n"
        );
    }

    #[test]
    fn a_replaced_standalone_output_is_backed_up() {
        let root = tempfile::tempdir().unwrap();
        let home = root.path().join("home");
        let repo = root.path().join("repo");
        let file = repo.join("home/.zprofile.luadot");
        write(&file, "generated\n");
        write(&home.join(".zprofile"), "handwritten\n");

        let saved = root.path().join("backup");
        let mut run = Run::new(false, Some(Backup::at("alt", &home, saved.clone())));
        resolve(
            &Config::default(),
            &home,
            &repo,
            &Entry::Standalone(file),
            &Classes::default(),
            &mut run,
        )
        .unwrap();

        assert_eq!(
            std::fs::read_to_string(home.join(".zprofile")).unwrap(),
            "generated\n"
        );
        assert_eq!(
            std::fs::read_to_string(saved.join("home/.zprofile")).unwrap(),
            "handwritten\n"
        );
    }

    #[test]
    fn a_dry_run_reports_a_standalone_template_and_touches_nothing() {
        let root = tempfile::tempdir().unwrap();
        let home = root.path().join("home");
        let repo = root.path().join("repo");
        let file = repo.join("home/.zprofile.luadot");
        write(&file, "generated\n");

        let mut run = Run::new(true, None);
        let outcomes = resolve(
            &Config::default(),
            &home,
            &repo,
            &Entry::Standalone(file),
            &Classes::default(),
            &mut run,
        )
        .unwrap();

        assert_eq!(outcomes, vec![SyncOutcome::Created]);
        assert!(!home.join(".zprofile").exists());
    }

    #[test]
    fn a_standalone_template_is_reached_through_the_path_it_produces() {
        let root = tempfile::tempdir().unwrap();
        let home = root.path().join("home");
        let repo = root.path().join("repo");
        let file = repo.join("home/.zprofile.luadot");
        write(&file, "generated\n");

        let arg = home.join(".zprofile").to_string_lossy().into_owned();

        assert_eq!(template_root(&home, &repo, &arg).unwrap(), file);
    }

    fn mode(path: &Path) -> u32 {
        use std::os::unix::fs::PermissionsExt;

        std::fs::metadata(path).unwrap().permissions().mode() & 0o7777
    }

    fn resolved(home: &Path, repo: &Path, dir: &Path, run: &mut Run) -> Vec<SyncOutcome> {
        resolve(
            &Config::default(),
            home,
            repo,
            &Entry::Template(dir.to_path_buf()),
            &Classes::default(),
            run,
        )
        .unwrap()
    }

    #[test]
    fn a_declared_mode_lands_on_the_generated_file() {
        let root = tempfile::tempdir().unwrap();
        let home = root.path().join("home");
        let repo = root.path().join("repo");
        let dir = repo.join("home/.netrc.luadot");
        write(
            &dir.join("luadot.lua"),
            r#"return { content = "machine example\n", mode = "600" }"#,
        );

        let outcomes = resolved(&home, &repo, &dir, &mut Run::default());

        assert_eq!(outcomes, vec![SyncOutcome::Created]);
        assert_eq!(mode(&home.join(".netrc")), 0o600);
    }

    #[test]
    fn a_file_whose_mode_diverges_is_written_again() {
        use std::os::unix::fs::PermissionsExt;

        let root = tempfile::tempdir().unwrap();
        let home = root.path().join("home");
        let repo = root.path().join("repo");
        let dir = repo.join("home/.netrc.luadot");
        write(
            &dir.join("luadot.lua"),
            r#"return { content = "machine example\n", mode = "600" }"#,
        );
        write(&home.join(".netrc"), "machine example\n");
        std::fs::set_permissions(home.join(".netrc"), std::fs::Permissions::from_mode(0o644))
            .unwrap();

        let outcomes = resolved(&home, &repo, &dir, &mut Run::default());

        assert_eq!(outcomes, vec![SyncOutcome::Replaced]);
        assert_eq!(mode(&home.join(".netrc")), 0o600);

        let outcomes = resolved(&home, &repo, &dir, &mut Run::default());

        assert_eq!(outcomes, vec![SyncOutcome::AlreadySynced]);
    }

    fn resolved_with(
        config: &Config,
        home: &Path,
        repo: &Path,
        dir: &Path,
        run: &mut Run,
    ) -> Vec<SyncOutcome> {
        let outcomes = resolve(
            config,
            home,
            repo,
            &Entry::Template(dir.to_path_buf()),
            &Classes::default(),
            run,
        )
        .unwrap();
        run.finish("alt").unwrap();

        outcomes
    }

    #[test]
    fn a_command_runs_only_when_the_file_changes() {
        let root = tempfile::tempdir().unwrap();
        let home = root.path().join("home");
        let repo = root.path().join("repo");
        let dir = repo.join("home/.config/mako/config.luadot");
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

    #[test]
    fn a_rule_of_the_configuration_names_the_command() {
        let root = tempfile::tempdir().unwrap();
        let home = root.path().join("home");
        let repo = root.path().join("repo");
        let dir = repo.join("home/.config/mako/config.luadot");
        let restarted = root.path().join("restarted");
        write(&dir.join("luadot.lua"), r#"return "font=monospace\n""#);

        let config = lua::from_source(&format!(
            r#"ld.rules({{ {{ match = "home/.config/mako/**", on_change = "printf ok > {}" }} }})"#,
            restarted.display()
        ))
        .unwrap();

        let outcomes = resolved_with(&config, &home, &repo, &dir, &mut Run::default());

        assert_eq!(outcomes, vec![SyncOutcome::Created]);
        assert_eq!(std::fs::read_to_string(&restarted).unwrap(), "ok");
    }

    #[test]
    fn a_declared_command_wins_over_the_rule() {
        let root = tempfile::tempdir().unwrap();
        let home = root.path().join("home");
        let repo = root.path().join("repo");
        let dir = repo.join("home/.config/mako/config.luadot");
        let declared = root.path().join("declared");
        let ruled = root.path().join("ruled");
        write(
            &dir.join("luadot.lua"),
            &format!(
                r#"return {{ content = "font=monospace\n", on_change = "printf ok > {}" }}"#,
                declared.display()
            ),
        );

        let config = lua::from_source(&format!(
            r#"ld.rules({{ {{ match = "home/.config/mako/**", on_change = "printf ok > {}" }} }})"#,
            ruled.display()
        ))
        .unwrap();

        resolved_with(&config, &home, &repo, &dir, &mut Run::default());

        assert!(declared.exists());
        assert!(!ruled.exists());
    }

    #[test]
    fn a_command_that_fails_stops_the_run() {
        let root = tempfile::tempdir().unwrap();
        let home = root.path().join("home");
        let repo = root.path().join("repo");
        let dir = repo.join("home/.zshrc.luadot");
        write(
            &dir.join("luadot.lua"),
            r#"return { content = "generated\n", on_change = "exit 4" }"#,
        );

        let mut run = Run::default();
        resolve(
            &Config::default(),
            &home,
            &repo,
            &Entry::Template(dir),
            &Classes::default(),
            &mut run,
        )
        .unwrap();

        let err = run.finish("alt").unwrap_err().to_string();

        assert_eq!(err, "alt: `exit 4` exited with status 4");
        assert!(home.join(".zshrc").exists());
    }

    #[test]
    fn a_dry_run_never_runs_the_command() {
        let root = tempfile::tempdir().unwrap();
        let home = root.path().join("home");
        let repo = root.path().join("repo");
        let dir = repo.join("home/.zshrc.luadot");
        let restarted = root.path().join("restarted");
        write(
            &dir.join("luadot.lua"),
            &format!(
                r#"return {{ content = "generated\n", on_change = "printf ok > {}" }}"#,
                restarted.display()
            ),
        );

        let mut run = Run::new(true, None);
        let outcomes = resolved_with(&Config::default(), &home, &repo, &dir, &mut run);

        assert_eq!(outcomes, vec![SyncOutcome::Created]);
        assert!(!restarted.exists());
    }
}
