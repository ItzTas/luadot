use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use clap::Args;

use crate::files::{self, Entry, FileStatus, Mirror, Side, Tracked};
use crate::lua::{
    self, Config, Content, Diff, DiffCounts, DiffFile, DiffState, Output, Shared, Tool,
};
use crate::output::{self, Tone};
use crate::state::{self, Classes};
use crate::utils::{self, SYSTEM_TEXT_MODE, Workspace};

use super::super::constants::{
    CUSTOM_ENTRY, CUSTOM_RENDER, CUSTOM_SUMMARY, DIFF_ARGUMENTS, DIFF_PROGRAM, GENERATED_FILES,
    MANAGED_FILES,
};

#[derive(Debug, Args)]
pub struct DiffArgs {
    #[arg(
        value_name = "PATH",
        help = "Narrow the report to this file or directory"
    )]
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
    let Workspace {
        config: shared,
        home,
        repo,
    } = utils::workspace("diff")?;
    let config = utils::configured("diff", &shared)?;

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
        let shown = drifted.len();
        show(&config, Side::Repository, &drifted)?;
        summary(&config, Side::Repository, shown, files.len())?;
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
    drop(config);
    let produced = resolve(&home, &repo, &templates, &classes, &shared)?;
    let config = utils::configured("diff", &shared)?;
    let drifted = generated_items(&config, &home, &produced)?;
    let shown = drifted.len();
    show(&config, Side::Generated, &drifted)?;

    summary(&config, Side::Generated, shown, produced.len())
}

fn managed_items(config: &Config, home: &Path, repo: &Path, files: &[Entry]) -> Result<Vec<Item>> {
    let mut drifted = Vec::new();
    for file in files.iter().map(Entry::path) {
        let relative = utils::relative(repo, file);
        let dest = utils::system_path(home, repo, file)?;
        let status = files::file_status(config.placement(relative), file, &dest)
            .with_context(|| format!("diff: failed to inspect {}", dest.display()))?;

        if !shows(status) {
            continue;
        }
        drifted.push(Item {
            relative: relative.to_path_buf(),
            dest,
            contents: files::read_contents("diff", file)?,
            mode: Some(files::effective_mode("diff", file, config.mode(relative))?),
        });
    }

    Ok(drifted)
}

fn resolve(
    home: &Path,
    repo: &Path,
    templates: &[Entry],
    classes: &Classes,
    shared: &Shared,
) -> Result<Vec<Output>> {
    let mut produced = Vec::new();
    for entry in templates {
        produced.extend(utils::outputs("diff", home, repo, entry, classes, shared)?);
    }

    Ok(produced)
}

fn generated_items(config: &Config, home: &Path, produced: &[Output]) -> Result<Vec<Item>> {
    let mut drifted = Vec::new();
    for output in produced {
        let status = utils::output_status("diff", config, home, output)?;
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
    let mode = utils::output_placement(config, relative, output).mode();

    match output.content() {
        Content::Text(text) => Ok((text.as_bytes().to_vec(), mode)),
        Content::File(source) => Ok((
            files::read_contents("diff", source)?,
            Some(files::effective_mode("diff", source, mode)?),
        )),
    }
}

fn show(config: &Config, side: Side, drifted: &[Item]) -> Result<()> {
    if drifted.is_empty() {
        return Ok(());
    }

    let files = inspected(side, drifted)?;

    if let Some(custom) = config.diff().render() {
        utils::said(custom.shown(
            &what(CUSTOM_RENDER),
            files.iter().collect::<Vec<&DiffFile>>(),
        )?);
        return Ok(());
    }

    for file in &files {
        announced(config, file)?;
    }

    staged(config, side, &files)
}

fn inspected(side: Side, drifted: &[Item]) -> Result<Vec<DiffFile>> {
    let mut files = Vec::new();
    for item in drifted {
        let system = system_side(&item.dest)?;
        let file = DiffFile::new(
            item.relative.clone(),
            item.dest.clone(),
            side,
            state(item, &system),
        );

        files.push(match system {
            System::Absent | System::Other => {
                file.with_source(item.contents.clone(), item.mode.unwrap_or(SYSTEM_TEXT_MODE))
            }
            System::Holds(found, mode) => file
                .with_source(item.contents.clone(), item.mode.unwrap_or(mode))
                .with_system(found, mode),
        });
    }

    Ok(files)
}

fn state(item: &Item, system: &System) -> DiffState {
    match system {
        System::Absent => DiffState::Missing,
        System::Other => DiffState::Other,
        System::Holds(found, _) if *found == item.contents => DiffState::Mode,
        System::Holds(_, _) => DiffState::Differs,
    }
}

fn announced(config: &Config, file: &DiffFile) -> Result<()> {
    let Some(custom) = config.diff().entry() else {
        reported(file);
        return Ok(());
    };

    utils::said(custom.shown(&what(CUSTOM_ENTRY), file)?);

    Ok(())
}

fn reported(file: &DiffFile) {
    let path = file.path().display();
    match file.state() {
        DiffState::Other => output::entry(Tone::Warning, "not a file", path),
        DiffState::Mode => output::entry(
            Tone::Warning,
            "mode",
            format!(
                "{path} {:04o} -> {:04o}",
                file.found_mode().unwrap_or(file.mode()),
                file.mode()
            ),
        ),
        DiffState::Missing | DiffState::Differs => {}
    }
}

fn staged(config: &Config, side: Side, files: &[DiffFile]) -> Result<()> {
    let staging: Vec<&DiffFile> = files.iter().filter(|file| file.state().staged()).collect();
    if staging.is_empty() {
        return Ok(());
    }

    let mirror = Mirror::open("diff")?;

    let Some(tool) = config.diff().tool() else {
        let tree = tracked_tree(&mirror, &staging)?;
        return run(tree.root(), DIFF_PROGRAM, &asked(config.diff()));
    };

    for file in &staging {
        mirror.place(side, file.path(), file.content(), file.mode())?;
        if let (Some(found), Some(mode)) = (file.found(), file.found_mode()) {
            mirror.place(Side::System, file.path(), found, mode)?;
        }
    }

    run(
        mirror.root(),
        tool.program(),
        &invocation(tool, config.diff(), side),
    )
}

fn tracked_tree<'a>(mirror: &'a Mirror, staging: &[&DiffFile]) -> Result<Tracked<'a>> {
    let mut tree = mirror.tracked()?;
    for file in staging {
        tree.write(file.path(), file.content(), file.mode())?;
    }
    tree.stage()?;

    for file in staging {
        held(&mut tree, file)?;
    }

    Ok(tree)
}

fn held(tree: &mut Tracked<'_>, file: &DiffFile) -> Result<()> {
    let (Some(found), Some(mode)) = (file.found(), file.found_mode()) else {
        return tree.erase(file.path());
    };

    tree.write(file.path(), found, mode)
}

fn asked(diff: &Diff) -> Vec<String> {
    let mut arguments: Vec<String> = DIFF_ARGUMENTS.iter().map(|word| word.to_string()).collect();
    arguments.extend(diff.args().iter().cloned());

    arguments
}

fn invocation(tool: &Tool, diff: &Diff, side: Side) -> Vec<String> {
    let mut arguments = tool.arguments().to_vec();
    arguments.extend(diff.args().iter().cloned());
    arguments.push(side.dir().to_string());
    arguments.push(Side::System.dir().to_string());

    arguments
}

fn summary(config: &Config, side: Side, drifted: usize, total: usize) -> Result<()> {
    let default = format!("{drifted} of {total} {} file(s) differ", named(side));

    let Some(custom) = config.diff().summary() else {
        output::note(default);
        return Ok(());
    };

    let counts = DiffCounts::new(side, drifted, total, default);

    utils::said(custom.shown(&what(CUSTOM_SUMMARY), &counts)?);

    Ok(())
}

fn named(side: Side) -> &'static str {
    match side {
        Side::Generated => GENERATED_FILES,
        Side::Repository | Side::System => MANAGED_FILES,
    }
}

fn what(key: &str) -> String {
    utils::customized("diff", &lua::Command::Diff.call(), key)
}

fn system_side(dest: &Path) -> Result<System> {
    let Ok(meta) = std::fs::metadata(dest) else {
        return Ok(System::Absent);
    };
    if !meta.is_file() {
        return Ok(System::Other);
    }

    let mode = files::effective_mode("diff", dest, None)?;

    Ok(System::Holds(files::read_contents("diff", dest)?, mode))
}

fn run(root: &Path, program: &str, arguments: &[String]) -> Result<()> {
    let status = build_command(root, program, arguments)
        .status()
        .with_context(|| format!("diff: failed to run {program}; is it installed and on PATH?"))?;

    match status.code() {
        None | Some(0 | 1) => Ok(()),
        Some(code) => bail!("diff: {program} exited with status {code}"),
    }
}

fn build_command(root: &Path, program: &str, arguments: &[String]) -> Command {
    let mut command = Command::new(program);
    command.current_dir(root);
    command.args(arguments);
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

    use super::*;

    fn arguments(command: &Command) -> Vec<String> {
        command
            .get_args()
            .map(|argument| argument.to_string_lossy().to_string())
            .collect()
    }

    fn tracked(name: &str) -> Command {
        let mut command = Command::new("git");
        command.args(["show", &format!(":{name}")]);
        command
    }

    #[test]
    fn git_is_asked_for_the_diff_it_always_prints() {
        let command = build_command(
            Path::new("/tmp/luadot-diff-1-0/tree"),
            DIFF_PROGRAM,
            &asked(&Diff::default()),
        );

        assert_eq!(command.get_program(), OsStr::new("git"));
        assert_eq!(
            command.get_current_dir(),
            Some(Path::new("/tmp/luadot-diff-1-0/tree"))
        );
        assert_eq!(arguments(&command), ["diff"]);
    }

    #[test]
    fn the_repository_side_is_staged_and_the_system_side_is_what_the_tree_holds() {
        let mirror = Mirror::open("diff").unwrap();
        let files = [
            DiffFile::new(
                PathBuf::from(".bashrc"),
                PathBuf::from("/home/u/.bashrc"),
                Side::Repository,
                DiffState::Differs,
            )
            .with_source(b"managed\n".to_vec(), 0o644)
            .with_system(b"handwritten\n".to_vec(), 0o644),
            DiffFile::new(
                PathBuf::from(".vimrc"),
                PathBuf::from("/home/u/.vimrc"),
                Side::Repository,
                DiffState::Missing,
            )
            .with_source(b"set number\n".to_vec(), 0o644),
        ];

        let tree = tracked_tree(&mirror, &files.iter().collect::<Vec<&DiffFile>>()).unwrap();

        let staged = tracked(".bashrc")
            .current_dir(tree.root())
            .output()
            .unwrap();
        assert_eq!(staged.stdout, b"managed\n");

        assert_eq!(
            std::fs::read(tree.root().join(".bashrc")).unwrap(),
            b"handwritten\n"
        );

        let staged = tracked(".vimrc").current_dir(tree.root()).output().unwrap();
        assert_eq!(staged.stdout, b"set number\n");
        assert!(!tree.root().join(".vimrc").exists());
    }

    #[test]
    fn a_tool_of_its_own_replaces_git_and_keeps_the_two_sides_last() {
        let tool = Tool::new("difft".to_string(), vec!["--color".to_string()]);
        let diff = Diff::default()
            .with_tool(Some(tool.clone()))
            .with_args(Some(vec!["always".to_string()]));

        let command = build_command(
            Path::new("/tmp/luadot-diff-1-0"),
            tool.program(),
            &invocation(&tool, &diff, Side::Repository),
        );

        assert_eq!(command.get_program(), OsStr::new("difft"));
        assert_eq!(
            arguments(&command),
            ["--color", "always", "repository", "system"]
        );
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
        let relative = Path::new(".netrc");

        let (contents, mode) = expected(&Config::default(), relative, &output).unwrap();
        assert_eq!(contents, b"machine example\n");
        assert_eq!(mode, None);

        let (_, mode) =
            expected(&Config::default(), relative, &output.with_mode(Some(0o600))).unwrap();
        assert_eq!(mode, Some(0o600));
    }
}
