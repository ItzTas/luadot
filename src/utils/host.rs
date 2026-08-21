use std::env;
use std::path::Path;

use super::constants::{HOSTNAME_FILES, HOSTNAME_VAR};

pub fn host_name() -> String {
    HOSTNAME_FILES
        .iter()
        .find_map(|path| read(Path::new(path)))
        .or_else(|| env::var(HOSTNAME_VAR).ok().and_then(trimmed))
        .unwrap_or_default()
}

fn read(path: &Path) -> Option<String> {
    trimmed(std::fs::read_to_string(path).ok()?)
}

fn trimmed(value: String) -> Option<String> {
    let value = value.trim();

    (!value.is_empty()).then(|| value.to_string())
}
