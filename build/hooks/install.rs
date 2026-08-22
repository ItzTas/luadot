use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

use super::constants::{CONFIG_KEY, DIR};

pub fn install() {
    let Ok(manifest) = env::var("CARGO_MANIFEST_DIR") else {
        return;
    };

    let manifest = PathBuf::from(manifest);

    if !manifest.join(DIR).is_dir() {
        return;
    }

    let Ok(root) = manifest.canonicalize() else {
        return;
    };

    if toplevel(&manifest) != Some(root) {
        return;
    }

    let _ = Command::new("git")
        .current_dir(&manifest)
        .args(["config", CONFIG_KEY, DIR])
        .status();
}

fn toplevel(dir: &Path) -> Option<PathBuf> {
    let output = Command::new("git")
        .current_dir(dir)
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let path = String::from_utf8(output.stdout).ok()?;

    Path::new(path.trim()).canonicalize().ok()
}
