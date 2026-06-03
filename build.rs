use std::process::Command;

fn main() {
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=.githooks");

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
