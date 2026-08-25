use std::path::Path;
use std::process::Command;

use tempfile::TempDir;

pub fn repository() -> TempDir {
    let repo = tempfile::tempdir().unwrap();

    for args in [
        vec!["init", "--quiet"],
        vec!["config", "user.email", "test@luadot"],
        vec!["config", "user.name", "luadot"],
        vec!["config", "commit.gpgsign", "false"],
    ] {
        let status = Command::new("git")
            .current_dir(repo.path())
            .args(args)
            .status()
            .unwrap();
        assert!(status.success());
    }

    repo
}

pub fn stage(repo: &Path, name: &str) {
    std::fs::write(repo.join(name), "contents\n").unwrap();
    let status = Command::new("git")
        .current_dir(repo)
        .args(["add", name])
        .status()
        .unwrap();
    assert!(status.success());
}
