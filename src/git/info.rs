use std::path::Path;

use anyhow::{Context, Result};
use tracing::debug;

use super::block::stitched;
use super::constants::{
    INFO_ATTRIBUTES, INFO_DIR, INFO_END, INFO_EXCLUDE, INFO_START, RULES_ATTRIBUTES, RULES_IGNORE,
};
use super::rules;

pub fn refresh(command: &str, repo: &Path) -> Result<bool> {
    let Ok(repository) = gix::open(repo) else {
        return Ok(false);
    };
    let Ok(rules) = rules::dir(command, repo) else {
        return Ok(false);
    };
    let info = repository.git_dir().join(INFO_DIR);

    let ignore = read(command, &rules.join(RULES_IGNORE))?;
    install(
        command,
        &info.join(INFO_EXCLUDE),
        ignore.as_deref().unwrap_or_default(),
    )?;

    let attributes = read(command, &rules.join(RULES_ATTRIBUTES))?;
    install(
        command,
        &info.join(INFO_ATTRIBUTES),
        attributes.as_deref().unwrap_or_default(),
    )?;

    Ok(attributes.is_some_and(|text| !text.trim().is_empty()))
}

fn read(command: &str, path: &Path) -> Result<Option<String>> {
    match std::fs::read_to_string(path) {
        Ok(text) => Ok(Some(text)),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(err) => {
            Err(err).with_context(|| format!("{command}: failed to read {}", path.display()))
        }
    }
}

fn install(command: &str, path: &Path, body: &str) -> Result<()> {
    let current = read(command, path)?.unwrap_or_default();
    let wanted = stitched(&current, INFO_START, INFO_END, body);
    if wanted == current {
        return Ok(());
    }

    debug!(path = %path.display(), "writing the rules of the repository for git");
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("{command}: failed to create {}", parent.display()))?;
    }
    std::fs::write(path, wanted)
        .with_context(|| format!("{command}: failed to write {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::fixture::repository;

    fn rule(repo: &Path, name: &str, contents: &str) {
        let dir = rules::dir("test", repo).unwrap();
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(name), contents).unwrap();
    }

    fn info(repo: &Path, name: &str) -> String {
        std::fs::read_to_string(repo.join(".git").join(INFO_DIR).join(name)).unwrap_or_default()
    }

    #[test]
    fn rules_land_between_markers() {
        let repo = repository();
        rule(repo.path(), RULES_IGNORE, "*.log\n.cache/\n");
        rule(repo.path(), RULES_ATTRIBUTES, "*.sh text eol=lf\n");

        assert!(refresh("add", repo.path()).unwrap());

        assert!(info(repo.path(), INFO_EXCLUDE).ends_with("# luadot\n*.log\n.cache/\n# /luadot\n"));
        assert_eq!(
            info(repo.path(), INFO_ATTRIBUTES),
            "# luadot\n*.sh text eol=lf\n# /luadot\n"
        );
    }

    #[test]
    fn handwritten_lines_are_kept() {
        let repo = repository();
        let exclude = repo.path().join(".git").join(INFO_DIR).join(INFO_EXCLUDE);
        std::fs::write(&exclude, "# by hand\nscratch/\n").unwrap();
        rule(repo.path(), RULES_IGNORE, "*.log\n");
        refresh("add", repo.path()).unwrap();

        rule(repo.path(), RULES_IGNORE, "*.tmp\n");
        refresh("add", repo.path()).unwrap();

        assert_eq!(
            info(repo.path(), INFO_EXCLUDE),
            "# by hand\nscratch/\n\n# luadot\n*.tmp\n# /luadot\n"
        );
    }

    #[test]
    fn a_missing_rule_file_drops_its_block() {
        let repo = repository();
        rule(repo.path(), RULES_IGNORE, "*.log\n");
        refresh("add", repo.path()).unwrap();
        std::fs::remove_file(rules::dir("test", repo.path()).unwrap().join(RULES_IGNORE)).unwrap();

        assert!(!refresh("add", repo.path()).unwrap());

        assert!(!info(repo.path(), INFO_EXCLUDE).contains("luadot"));
    }

    #[test]
    fn no_repository_means_no_write() {
        let dir = tempfile::tempdir().unwrap();
        rule(dir.path(), RULES_IGNORE, "*.log\n");

        assert!(!refresh("status", dir.path()).unwrap());

        assert!(!dir.path().join(".git").exists());
    }
}
