use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use tracing::debug;

use super::block::stitched;
use super::constants::{MARKER_END, MARKER_START, RULES_ATTRIBUTES, TRACKED, UNTRACKED};
use super::rules;

pub fn path(command: &str, repo: &Path) -> Result<PathBuf> {
    Ok(rules::dir(command, repo)?.join(RULES_ATTRIBUTES))
}

pub fn sync(command: &str, repo: &Path, patterns: &[(String, bool)]) -> Result<bool> {
    let path = path(command, repo)?;
    let current = match std::fs::read_to_string(&path) {
        Ok(current) => current,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(err) => {
            return Err(err)
                .with_context(|| format!("{command}: failed to read {}", path.display()));
        }
    };

    let wanted = rendered(&current, patterns);
    if wanted == current {
        return Ok(false);
    }

    debug!(path = %path.display(), patterns = patterns.len(), "writing the lfs attributes");
    if wanted.is_empty() {
        std::fs::remove_file(&path)
            .with_context(|| format!("{command}: failed to remove {}", path.display()))?;
        return Ok(true);
    }

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("{command}: failed to create {}", parent.display()))?;
    }
    std::fs::write(&path, &wanted)
        .with_context(|| format!("{command}: failed to write {}", path.display()))?;

    Ok(true)
}

fn rendered(current: &str, patterns: &[(String, bool)]) -> String {
    stitched(
        current,
        MARKER_START,
        MARKER_END,
        &attributes(patterns).join("\n"),
    )
}

fn attributes(patterns: &[(String, bool)]) -> Vec<String> {
    patterns
        .iter()
        .flat_map(|(pattern, tracked)| {
            expanded(pattern)
                .into_iter()
                .map(move |pattern| (pattern, *tracked))
        })
        .map(|(pattern, tracked)| match tracked {
            true => format!("{pattern} {TRACKED}"),
            false => format!("{pattern} {UNTRACKED}"),
        })
        .collect()
}

fn expanded(pattern: &str) -> Vec<String> {
    let anchored = match pattern.contains('/') {
        true => pattern.to_string(),
        false => format!("/{pattern}"),
    };
    if anchored.ends_with("**") {
        return vec![anchored];
    }

    let subtree = format!("{anchored}/**");

    vec![anchored, subtree]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tracked(patterns: &[&str]) -> Vec<(String, bool)> {
        patterns
            .iter()
            .map(|pattern| ((*pattern).to_string(), true))
            .collect()
    }

    #[test]
    fn a_pattern_naming_a_directory_carries_its_subtree() {
        assert_eq!(
            expanded(".config/nvim"),
            [".config/nvim", ".config/nvim/**"]
        );
        assert_eq!(expanded("Videos/**"), ["Videos/**"]);
    }

    #[test]
    fn the_block_keeps_the_lines_written_by_hand() {
        let current = "* text=auto\n.claude/settings.json merge=claude-settings\n";

        let rendered = rendered(current, &tracked(&["Videos/**"]));

        assert_eq!(
            rendered,
            "* text=auto\n.claude/settings.json merge=claude-settings\n\n# luadot:lfs\nVideos/** filter=lfs diff=lfs merge=lfs -text\n# /luadot:lfs\n"
        );
    }

    #[test]
    fn dropping_every_pattern_takes_the_block_out_again() {
        let written = rendered("* text=auto\n", &tracked(&["Videos/**"]));

        assert_eq!(rendered(&written, &[]), "* text=auto\n");
    }
}
