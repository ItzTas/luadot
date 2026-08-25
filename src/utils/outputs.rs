use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

use super::paths::managed_relative;
use crate::files::{self, Entry, FileStatus, Placement};
use crate::lua::{self, Config, Content, Output, Shared};
use crate::state::Classes;

pub fn outputs(
    command: &str,
    home: &Path,
    repo: &Path,
    entry: &Entry,
    classes: &Classes,
    config: &Shared,
) -> Result<Vec<Output>> {
    match entry {
        Entry::Template(dir) => lua::load_template(command, home, repo, dir, classes, config),
        Entry::Standalone(path) => Ok(vec![lua::load_template_file(
            command, home, repo, path, classes, config,
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
    let dest = output.dest();
    let relative = output_relative(command, home, output)?;
    let placement = output_placement(config, &relative, output);

    match output.content() {
        Content::File(source) => files::file_status(placement, source, dest),
        Content::Text(text) => files::text_status(dest, text, placement.mode()),
    }
    .with_context(|| format!("{command}: failed to inspect {}", dest.display()))
}

pub fn output_relative(command: &str, home: &Path, output: &Output) -> Result<PathBuf> {
    managed_relative(home, output.dest())
        .with_context(|| format!("{command}: failed to place {}", output.dest().display()))
}

pub fn output_placement<'a>(
    config: &'a Config,
    relative: &'a Path,
    output: &Output,
) -> Placement<'a> {
    let placement = config.placement(relative);

    placement
        .with_link(output.link().unwrap_or_else(|| placement.link()))
        .with_mode(output.mode().or_else(|| placement.mode()))
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    fn configuration() -> Shared {
        Arc::new(Mutex::new(Config::default()))
    }

    use super::*;
    use crate::files::LinkMode;

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
    fn a_template_resolves_into_its_files() {
        let root = tempfile::tempdir().unwrap();
        let home = root.path().join("home");
        let repo = root.path().join("repo");
        let dir = repo.join(".zshrc.luadot");
        write(&dir.join("luadot.lua"), r#"return "generated\n""#);

        let resolved = outputs(
            "status",
            &home,
            &repo,
            &Entry::Template(dir),
            &Classes::default(),
            &configuration(),
        )
        .unwrap();

        assert_eq!(resolved.len(), 1);
        assert_eq!(resolved[0].dest(), home.join(".zshrc"));
    }

    #[test]
    fn generated_content_is_compared() {
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
    fn a_destination_outside_home_fails() {
        let err = output_relative(
            "tmpl alt",
            Path::new("/home/u"),
            &text(PathBuf::from("/etc/motd"), "welcome\n"),
        )
        .unwrap_err();

        assert_eq!(
            format!("{err:#}"),
            "tmpl alt: failed to place /etc/motd: outside your home directory /home/u"
        );
    }

    #[test]
    fn the_output_wins_over_the_rules() {
        let config = lua::from_source(
            r#"ld.rules({ match = ".netrc", link = "copy", mode = "0640", owner = "me" })"#,
        )
        .unwrap();
        let relative = Path::new(".netrc");
        let output = text(PathBuf::from("/home/u/.netrc"), "x");

        let placement = output_placement(&config, relative, &output);
        assert_eq!(placement.link(), LinkMode::Copy);
        assert_eq!(placement.mode(), Some(0o640));
        assert_eq!(placement.owner(), Some("me"));

        let declared = Output::new(
            PathBuf::from("/home/u/.netrc"),
            Content::Text("x".to_string()),
            Some(LinkMode::Symbolic),
            None,
        )
        .with_mode(Some(0o600));
        let placement = output_placement(&config, relative, &declared);
        assert_eq!(placement.link(), LinkMode::Symbolic);
        assert_eq!(placement.mode(), Some(0o600));
    }
}
