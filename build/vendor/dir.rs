use std::env;
use std::error::Error;
use std::path::PathBuf;

const DIR: &str = "vendor";

pub fn dir(name: &str) -> Result<PathBuf, Box<dyn Error>> {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let source = manifest.join(DIR).join(name);
    if !source.is_dir() {
        return Err(format!("nothing vendored at {}", source.display()).into());
    }

    println!("cargo::rerun-if-changed={}", source.display());

    Ok(source)
}
