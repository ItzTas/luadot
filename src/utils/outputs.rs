use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

use super::constants::SYSTEM_TEXT_MODE;
use super::paths::{is_root, managed_relative};
use crate::files::{self, Entry, FileStatus};
use crate::lua::{self, Config, Content, Output};
use crate::state::Classes;

type Inspect = fn(&Path, &Path, Option<u32>) -> Result<FileStatus>;

pub fn outputs(
    command: &str,
    home: &Path,
    repo: &Path,
    entry: &Entry,
    classes: &Classes,
) -> Result<Vec<Output>> {
    match entry {
        Entry::Template(dir) => lua::load_template(command, home, repo, dir, classes),
        Entry::Standalone(path) => Ok(vec![lua::load_template_file(
            command, home, repo, path, classes,
        )?]),
        Entry::File(path) => bail!("{command}: {} is not a template", path.display()),
    }
}

pub fn output_status(
    command: &str,
    config: &Config,
    home: &Path,
    output: &Output,
) -> Result<FileStatus> {
    inspected(files::inspect_system, command, config, home, output)
}

pub fn escalated_output_status(
    command: &str,
    config: &Config,
    home: &Path,
    output: &Output,
) -> Result<FileStatus> {
    inspected(files::escalated_status, command, config, home, output)
}

pub fn output_relative(command: &str, home: &Path, output: &Output) -> Result<PathBuf> {
    managed_relative(home, output.dest())
        .with_context(|| format!("{command}: failed to place {}", output.dest().display()))
}

pub fn generated_mode(config: &Config, relative: &Path, output: &Output) -> Option<u32> {
    if is_root(relative) {
        return Some(
            output
                .mode()
                .or_else(|| config.mode(relative))
                .unwrap_or(SYSTEM_TEXT_MODE),
        );
    }

    output.mode()
}

fn inspected(
    inspect: Inspect,
    command: &str,
    config: &Config,
    home: &Path,
    output: &Output,
) -> Result<FileStatus> {
    let dest = output.dest();
    let relative = output_relative(command, home, output)?;
    let failed = || format!("{command}: failed to inspect {}", dest.display());

    if is_root(&relative) {
        return root_status(inspect, config, &relative, output).with_context(failed);
    }

    let mode = output.link().unwrap_or_else(|| config.link_mode(&relative));
    match output.content() {
        Content::File(source) => files::file_status(mode, source, dest),
        Content::Text(text) => files::text_status(dest, text, output.mode()),
    }
    .with_context(failed)
}

fn root_status(
    inspect: Inspect,
    config: &Config,
    relative: &Path,
    output: &Output,
) -> Result<FileStatus> {
    let staged;
    let (source, mode) = match output.content() {
        Content::File(source) => (source.as_path(), config.mode(relative)),
        Content::Text(text) => {
            staged = files::stage_text(text)?;
            (staged.path(), generated_mode(config, relative, output))
        }
    };

    inspect(source, output.dest(), mode)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::files::{ConflictPolicy, LinkMode};

    fn write(path: &Path, contents: &str) {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(path, contents).unwrap();
    }

    fn text(dest: PathBuf, contents: &str) -> Output {
        Output::new(dest, Content::Text(contents.to_string()), None, None)
    }

    #[test]
    fn a_template_directory_resolves_into_the_files_it_declares() {
        let root = tempfile::tempdir().unwrap();
        let home = root.path().join("home");
        let repo = root.path().join("repo");
        let dir = repo.join("home/.zshrc.luadot");
        write(&dir.join("luadot.lua"), r#"return "generated\n""#);

        let resolved = outputs(
            "status",
            &home,
            &repo,
            &Entry::Template(dir),
            &Classes::default(),
        )
        .unwrap();

        assert_eq!(resolved.len(), 1);
        assert_eq!(resolved[0].dest(), home.join(".zshrc"));
    }

    #[test]
    fn a_standalone_template_resolves_into_one_file() {
        let root = tempfile::tempdir().unwrap();
        let home = root.path().join("home");
        let repo = root.path().join("repo");
        let file = repo.join("home/.zprofile.luadot");
        write(&file, "export HOST=<%= 1 + 1 %>\n");

        let resolved = outputs(
            "diff",
            &home,
            &repo,
            &Entry::Standalone(file),
            &Classes::default(),
        )
        .unwrap();

        assert_eq!(
            resolved[0].content(),
            &Content::Text("export HOST=2\n".to_string())
        );
    }

    #[test]
    fn a_plain_file_is_not_a_template() {
        let err = outputs(
            "status",
            Path::new("/home/u"),
            Path::new("/repo"),
            &Entry::File(PathBuf::from("/repo/home/.vimrc")),
            &Classes::default(),
        )
        .unwrap_err()
        .to_string();

        assert!(err.contains("status: "));
        assert!(err.contains("is not a template"));
    }

    #[test]
    fn generated_content_is_compared_against_what_the_system_holds() {
        let root = tempfile::tempdir().unwrap();
        let home = root.path().join("home");
        let dest = home.join(".zshrc");
        let output = text(dest.clone(), "generated\n");

        assert_eq!(
            output_status("status", &Config::default(), &home, &output).unwrap(),
            FileStatus::Missing
        );

        write(&dest, "generated\n");
        assert_eq!(
            output_status("status", &Config::default(), &home, &output).unwrap(),
            FileStatus::Synced
        );

        write(&dest, "handwritten\n");
        assert_eq!(
            output_status("status", &Config::default(), &home, &output).unwrap(),
            FileStatus::Differs
        );
    }

    #[test]
    fn a_selected_file_is_compared_through_the_link_mode() {
        let root = tempfile::tempdir().unwrap();
        let home = root.path().join("home");
        let source = root.path().join("repo/home/.zshrc.luadot/laptop.zsh");
        let dest = home.join(".zshrc");
        write(&source, "laptop\n");
        std::fs::create_dir_all(&home).unwrap();
        std::os::unix::fs::symlink(&source, &dest).unwrap();

        let output = Output::new(
            dest,
            Content::File(source),
            Some(LinkMode::Symbolic),
            Some(ConflictPolicy::Overwrite),
        );

        assert_eq!(
            output_status("status", &Config::default(), &home, &output).unwrap(),
            FileStatus::Synced
        );
    }

    #[test]
    fn a_declared_mode_is_part_of_the_comparison() {
        use std::os::unix::fs::PermissionsExt;

        let root = tempfile::tempdir().unwrap();
        let home = root.path().join("home");
        let dest = home.join(".netrc");
        write(&dest, "machine example\n");
        std::fs::set_permissions(&dest, std::fs::Permissions::from_mode(0o644)).unwrap();

        let output = text(dest, "machine example\n").with_mode(Some(0o600));

        assert_eq!(
            output_status("status", &Config::default(), &home, &output).unwrap(),
            FileStatus::Differs
        );
    }

    #[test]
    fn a_generated_system_file_falls_back_to_the_system_mode() {
        let config = Config::default();
        let relative = Path::new("root/etc/motd");
        let output = text(PathBuf::from("/etc/motd"), "welcome\n");

        assert_eq!(
            generated_mode(&config, relative, &output),
            Some(SYSTEM_TEXT_MODE)
        );
        assert_eq!(
            generated_mode(&config, relative, &output.clone().with_mode(Some(0o600))),
            Some(0o600)
        );
    }

    #[test]
    fn a_generated_home_file_keeps_the_mode_it_declares() {
        let config = Config::default();
        let relative = Path::new("home/.netrc");
        let output = text(PathBuf::from("/home/u/.netrc"), "machine example\n");

        assert_eq!(generated_mode(&config, relative, &output), None);
        assert_eq!(
            generated_mode(&config, relative, &output.with_mode(Some(0o600))),
            Some(0o600)
        );
    }

    #[test]
    fn the_relative_path_of_an_output_mirrors_the_repository() {
        let home = Path::new("/home/u");

        assert_eq!(
            output_relative("status", home, &text(PathBuf::from("/home/u/.zshrc"), "x")).unwrap(),
            PathBuf::from("home/.zshrc")
        );
        assert_eq!(
            output_relative("status", home, &text(PathBuf::from("/etc/motd"), "x")).unwrap(),
            PathBuf::from("root/etc/motd")
        );
    }
}
