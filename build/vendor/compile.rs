use std::collections::BTreeSet;
use std::error::Error;
use std::path::{Path, PathBuf};

use super::dir::dir;
use super::sources::sources;

pub fn compile(name: &str, headers: &Path) -> Result<PathBuf, Box<dyn Error>> {
    let source = dir(name)?;
    let files = sources(&source)?;
    if files.is_empty() {
        return Err(format!("no source to compile in {}", source.display()).into());
    }

    let mut includes = BTreeSet::from([source.clone()]);
    for file in &files {
        if let Some(parent) = file.parent() {
            includes.insert(parent.to_path_buf());
        }
    }

    let mut build = cc::Build::new();
    build.include(headers);
    for include in &includes {
        build.include(include);
    }
    build.files(&files);
    build.compile(name);

    Ok(source)
}
