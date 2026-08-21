use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use tracing::debug;

use super::constants::{ATTRIBUTES_FILE, MARKER_END, MARKER_START, TRACKED, UNTRACKED};

pub fn path(repo: &Path) -> PathBuf {
    repo.join(ATTRIBUTES_FILE)
}

pub fn sync(command: &str, repo: &Path, patterns: &[(String, bool)]) -> Result<bool> {
    let path = path(repo);
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

    std::fs::write(&path, &wanted)
        .with_context(|| format!("{command}: failed to write {}", path.display()))?;

    Ok(true)
}

fn rendered(current: &str, patterns: &[(String, bool)]) -> String {
    let kept = outside(current);
    let block = block(patterns);

    match (kept.is_empty(), block.is_empty()) {
        (true, true) => String::new(),
        (true, false) => block,
        (false, true) => format!("{kept}\n"),
        (false, false) => format!("{kept}\n\n{block}"),
    }
}

fn outside(current: &str) -> String {
    let mut kept: Vec<&str> = Vec::new();
    let mut inside = false;

    for line in current.lines() {
        if line.trim() == MARKER_START {
            inside = true;
            continue;
        }
        if line.trim() == MARKER_END {
            inside = false;
            continue;
        }
        if !inside {
            kept.push(line);
        }
    }

    while kept.last().is_some_and(|line| line.trim().is_empty()) {
        kept.pop();
    }

    kept.join("\n")
}

fn block(patterns: &[(String, bool)]) -> String {
    let lines = attributes(patterns);
    if lines.is_empty() {
        return String::new();
    }

    format!("{MARKER_START}\n{}\n{MARKER_END}\n", lines.join("\n"))
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
    fn a_pattern_without_a_separator_is_anchored_at_the_root() {
        assert_eq!(expanded("*.mp4"), ["/*.mp4", "/*.mp4/**"]);
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
    fn the_block_is_replaced_instead_of_stacked() {
        let first = rendered("", &tracked(&["Videos/**"]));

        let second = rendered(&first, &tracked(&["Pictures/**"]));

        assert_eq!(
            second,
            "# luadot:lfs\nPictures/** filter=lfs diff=lfs merge=lfs -text\n# /luadot:lfs\n"
        );
    }

    #[test]
    fn a_pattern_taken_back_unsets_the_attributes() {
        let rendered = rendered(
            "",
            &[
                ("Videos/**".to_string(), true),
                ("Videos/notes/**".to_string(), false),
            ],
        );

        assert_eq!(
            rendered,
            "# luadot:lfs\nVideos/** filter=lfs diff=lfs merge=lfs -text\nVideos/notes/** -filter -diff -merge text\n# /luadot:lfs\n"
        );
    }

    #[test]
    fn a_file_without_patterns_is_left_as_it_was() {
        let current = "* text=auto\n";

        assert_eq!(rendered(current, &[]), current);
    }

    #[test]
    fn dropping_every_pattern_takes_the_block_out_again() {
        let written = rendered("* text=auto\n", &tracked(&["Videos/**"]));

        assert_eq!(rendered(&written, &[]), "* text=auto\n");
    }

    #[test]
    fn syncing_writes_the_file_once_and_removes_it_when_nothing_is_left() {
        let repo = tempfile::tempdir().unwrap();

        assert!(sync("add", repo.path(), &tracked(&["Videos/**"])).unwrap());
        assert!(!sync("add", repo.path(), &tracked(&["Videos/**"])).unwrap());

        assert!(sync("add", repo.path(), &[]).unwrap());
        assert!(!path(repo.path()).exists());
    }
}
