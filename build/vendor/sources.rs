use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

const SOURCE_EXTENSION: &str = "c";

pub fn sources(dir: &Path) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let mut found = Vec::new();
    collect(dir, &mut found)?;
    found.sort();

    Ok(found)
}

fn collect(dir: &Path, found: &mut Vec<PathBuf>) -> Result<(), Box<dyn Error>> {
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            collect(&path, found)?;
            continue;
        }

        if path
            .extension()
            .is_some_and(|extension| extension == SOURCE_EXTENSION)
        {
            found.push(path);
        }
    }

    Ok(())
}
