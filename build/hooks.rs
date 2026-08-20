use std::process::Command;

pub fn install() {
    let inside_work_tree = Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .output()
        .map(|out| out.status.success() && out.stdout.starts_with(b"true"))
        .unwrap_or(false);

    if !inside_work_tree {
        return;
    }

    let _ = Command::new("git")
        .args(["config", "core.hooksPath", ".githooks"])
        .status();
}
