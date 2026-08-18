use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use clap::Args;

use crate::files::{self, Entry, FileStatus, Mirror, Side};
use crate::lua::{Config, Content, Output};
use crate::output::{self, Tone};
use crate::state::{self, Classes};
use crate::utils::{self, SYSTEM_TEXT_MODE, Workspace};

use super::super::constants::DIFF_ARGUMENTS;

#[derive(Debug, Args)]
pub struct DiffArgs {
    #[arg(value_name = "PATH")]
    pub path: Option<String>,
    #[arg(
        short,
        long,
        help = "Resolve the templates and diff the files they produce"
    )]
    pub templates: bool,
}

enum System {
    Absent,
    Other,
    Holds(Vec<u8>, u32),
}

struct Item {
    relative: PathBuf,
    dest: PathBuf,
    contents: Vec<u8>,
    mode: Option<u32>,
}

pub fn diff_cmd(args: DiffArgs) -> Result<()> {
    let Workspace { config, home, repo } = utils::workspace("diff")?;

    let root = utils::managed_root("diff", &home, &repo, args.path.as_deref())?;

    let (files, templates): (Vec<Entry>, Vec<Entry>) =
        utils::managed_entries("diff", &repo, &root, |relative| config.is_ignored(relative))?
            .into_iter()
            .partition(|entry| matches!(entry, Entry::File(_)));

    if files.is_empty() && templates.is_empty() {
        output::note("nothing is managed");
        return Ok(());
    }

    if !files.is_empty() {
        let drifted = managed_items(&config, &home, &repo, &files)?;
        show(Side::Repository, &drifted)?;
        output::note(format!(
            "{} of {} managed file(s) differ",
            drifted.len(),
            files.len()
        ));
    }

    if templates.is_empty() {
        return Ok(());
    }
    if !args.templates {
        output::note(format!(
            "{} template(s) skipped (run with --templates)",
            templates.len()
        ));
        return Ok(());
    }

    let classes = state::load()?.classes().clone();
    let produced = resolve(&home, &repo, &templates, &classes)?;
    let drifted = generated_items(&config, &home, &produced)?;
    show(Side::Generated, &drifted)?;
    output::note(format!(
        "{} of {} generated file(s) differ",
        drifted.len(),
        produced.len()
    ));

    Ok(())
}

fn managed_items(config: &Config, home: &Path, repo: &Path, files: &[Entry]) -> Result<Vec<Item>> {
    let mut drifted = Vec::new();
    for file in files.iter().map(Entry::path) {
        let relative = utils::relative(repo, file);
        let dest = utils::system_path(home, repo, file)?;
        let status = match utils::is_root(relative) {
            true => files::escalated_status(file, &dest, config.mode(relative)),
            false => files::file_status(config.link_mode(relative), file, &dest),
        }
        .with_context(|| format!("diff: failed to inspect {}", dest.display()))?;

        if !shows(status) {
            continue;
        }
        drifted.push(Item {
            relative: relative.to_path_buf(),
            dest,
            contents: files::read_contents("diff", file)?,
            mode: Some(files::effective_mode(file, config.mode(relative))?),
        });
    }

    Ok(drifted)
}

fn resolve(
    home: &Path,
    repo: &Path,
    templates: &[Entry],
    classes: &Classes,
) -> Result<Vec<Output>> {
    let mut produced = Vec::new();
    for entry in templates {
        produced.extend(utils::outputs("diff", home, repo, entry, classes)?);
    }

    Ok(produced)
}

fn generated_items(config: &Config, home: &Path, produced: &[Output]) -> Result<Vec<Item>> {
    let mut drifted = Vec::new();
    for output in produced {
        let status = utils::escalated_output_status("diff", config, home, output)?;
        if !shows(status) {
            continue;
        }

        let relative = utils::output_relative("diff", home, output)?;
        let (contents, mode) = expected(config, &relative, output)?;
        drifted.push(Item {
            relative,
            dest: output.dest().to_path_buf(),
            contents,
            mode,
        });
    }

    Ok(drifted)
}

fn expected(config: &Config, relative: &Path, output: &Output) -> Result<(Vec<u8>, Option<u32>)> {
    match output.content() {
        Content::Text(text) => Ok((
            text.as_bytes().to_vec(),
            utils::generated_mode(config, relative, output),
        )),
        Content::File(source) => Ok((
            files::read_contents("diff", source)?,
            Some(files::effective_mode(source, config.mode(relative))?),
        )),
    }
}

fn show(side: Side, drifted: &[Item]) -> Result<()> {
    if drifted.is_empty() {
        return Ok(());
    }

    let mirror = Mirror::open("diff")?;

    let mut staged = 0u32;
    for item in drifted {
        let relative = item.relative.as_path();

        match system_side(&item.dest)? {
            System::Other => output::entry(Tone::Warning, "not a file", relative.display()),
            System::Absent => {
                let expected = item.mode.unwrap_or(SYSTEM_TEXT_MODE);
                mirror.place(side, relative, &item.contents, expected)?;
                staged += 1;
            }
            System::Holds(found, mode) if found == item.contents => {
                let expected = item.mode.unwrap_or(mode);
                output::entry(
                    Tone::Warning,
                    "mode",
                    format!("{} {mode:04o} -> {expected:04o}", relative.display()),
                );
            }
            System::Holds(found, mode) => {
                mirror.place(side, relative, &item.contents, item.mode.unwrap_or(mode))?;
                mirror.place(Side::System, relative, &found, mode)?;
                staged += 1;
            }
        }
    }

    if staged == 0 {
        return Ok(());
    }

    run(mirror.root(), side)
}

fn system_side(dest: &Path) -> Result<System> {
    let Ok(meta) = std::fs::metadata(dest) else {
        return Ok(System::Absent);
    };
    if !meta.is_file() {
        return Ok(System::Other);
    }

    let mode = files::effective_mode(dest, None)?;

    Ok(System::Holds(files::read_contents("diff", dest)?, mode))
}

fn run(root: &Path, side: Side) -> Result<()> {
    let status = build_command(root, side)
        .status()
        .context("diff: failed to run git; is it installed and on PATH?")?;

    match status.code() {
        None | Some(0 | 1) => Ok(()),
        Some(code) => bail!("diff: git exited with status {code}"),
    }
}

fn build_command(root: &Path, side: Side) -> Command {
    let mut command = Command::new("git");
    command.current_dir(root);
    command.args(DIFF_ARGUMENTS);
    command.args([side.dir(), Side::System.dir()]);
    command
}

fn shows(status: FileStatus) -> bool {
    matches!(
        status,
        FileStatus::Missing | FileStatus::Differs | FileStatus::Unreadable
    )
}

#[cfg(test)]
mod tests {
    use std::ffi::OsStr;
    use std::os::unix::fs::PermissionsExt;

    use super::*;

    #[test]
    fn only_the_states_with_something_to_show_are_diffed() {
        assert!(shows(FileStatus::Missing));
        assert!(shows(FileStatus::Differs));
        assert!(shows(FileStatus::Unreadable));
        assert!(!shows(FileStatus::Synced));
        assert!(!shows(FileStatus::Unlinked));
    }

    #[test]
    fn git_compares_the_two_sides_from_inside_the_mirror() {
        let command = build_command(Path::new("/tmp/luadot-diff-1-0"), Side::Repository);

        assert_eq!(command.get_program(), OsStr::new("git"));
        assert_eq!(
            command.get_current_dir(),
            Some(Path::new("/tmp/luadot-diff-1-0"))
        );

        let args: Vec<&str> = command.get_args().map(|a| a.to_str().unwrap()).collect();
        assert_eq!(
            args,
            [
                "diff",
                "--no-index",
                "--no-prefix",
                "--",
                "repository",
                "system"
            ]
        );
    }

    #[test]
    fn what_a_template_produces_is_compared_from_its_own_side() {
        let command = build_command(Path::new("/tmp/luadot-diff-1-1"), Side::Generated);

        let args: Vec<&str> = command.get_args().map(|a| a.to_str().unwrap()).collect();
        assert_eq!(args.last(), Some(&"system"));
        assert_eq!(args[args.len() - 2], "generated");
    }

    #[test]
    fn the_system_side_is_read_only_when_it_is_a_file() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join(".bashrc");
        std::fs::write(&file, "handwritten\n").unwrap();
        std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o600)).unwrap();

        let System::Holds(contents, mode) = system_side(&file).unwrap() else {
            panic!("a regular file holds its contents");
        };
        assert_eq!(contents, b"handwritten\n");
        assert_eq!(mode, 0o600);

        assert!(matches!(
            system_side(&dir.path().join("gone")).unwrap(),
            System::Absent
        ));
        assert!(matches!(system_side(dir.path()).unwrap(), System::Other));

        let dangling = dir.path().join(".zshrc");
        std::os::unix::fs::symlink(dir.path().join("gone"), &dangling).unwrap();
        assert!(matches!(system_side(&dangling).unwrap(), System::Absent));
    }

    #[test]
    fn generated_content_carries_the_mode_the_template_declares() {
        let dest = PathBuf::from("/home/u/.netrc");
        let output = Output::new(
            dest.clone(),
            Content::Text("machine example\n".to_string()),
            None,
            None,
        );
        let relative = Path::new("home/.netrc");

        let (contents, mode) = expected(&Config::default(), relative, &output).unwrap();
        assert_eq!(contents, b"machine example\n");
        assert_eq!(mode, None);

        let (_, mode) =
            expected(&Config::default(), relative, &output.with_mode(Some(0o600))).unwrap();
        assert_eq!(mode, Some(0o600));
    }

    #[test]
    fn a_selected_file_is_read_from_the_repository_with_its_own_mode() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("laptop.zsh");
        std::fs::write(&source, "laptop\n").unwrap();
        std::fs::set_permissions(&source, std::fs::Permissions::from_mode(0o640)).unwrap();

        let output = Output::new(
            PathBuf::from("/home/u/.zshrc"),
            Content::File(source),
            None,
            None,
        );

        let (contents, mode) =
            expected(&Config::default(), Path::new("home/.zshrc"), &output).unwrap();

        assert_eq!(contents, b"laptop\n");
        assert_eq!(mode, Some(0o640));
    }

    #[test]
    fn a_drifted_generated_file_is_staged_on_both_sides() {
        let root = tempfile::tempdir().unwrap();
        let home = root.path().join("home");
        let repo = root.path().join("repo");
        let dir = repo.join("home/.zshrc.luadot");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::create_dir_all(&home).unwrap();
        std::fs::write(dir.join("luadot.lua"), r#"return "generated\n""#).unwrap();
        std::fs::write(home.join(".zshrc"), "handwritten\n").unwrap();

        let produced = resolve(&home, &repo, &[Entry::Template(dir)], &Classes::default()).unwrap();
        let drifted = generated_items(&Config::default(), &home, &produced).unwrap();

        assert_eq!(drifted.len(), 1);
        assert_eq!(drifted[0].relative, Path::new("home/.zshrc"));
        assert_eq!(drifted[0].contents, b"generated\n");
    }

    #[test]
    fn a_generated_file_the_system_already_holds_is_left_out() {
        let root = tempfile::tempdir().unwrap();
        let home = root.path().join("home");
        let repo = root.path().join("repo");
        let dir = repo.join("home/.zshrc.luadot");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::create_dir_all(&home).unwrap();
        std::fs::write(dir.join("luadot.lua"), r#"return "generated\n""#).unwrap();
        std::fs::write(home.join(".zshrc"), "generated\n").unwrap();

        let produced = resolve(&home, &repo, &[Entry::Template(dir)], &Classes::default()).unwrap();

        assert!(
            generated_items(&Config::default(), &home, &produced)
                .unwrap()
                .is_empty()
        );
    }
}
