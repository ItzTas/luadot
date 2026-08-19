use std::error::Error;
use std::path::Path;

use crate::vendor;

use super::constants::{NAME, RE_ENV, RE_FILE};

pub fn compile(headers: &Path) -> Result<(), Box<dyn Error>> {
    let source = vendor::compile(NAME, headers)?;

    let re = source.join(RE_FILE);
    if !re.is_file() {
        return Err(format!("no {RE_FILE} in {}", source.display()).into());
    }

    println!("cargo::rustc-env={RE_ENV}={}", re.display());

    Ok(())
}
