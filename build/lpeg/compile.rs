use std::error::Error;
use std::path::Path;

use crate::vendor;

const NAME: &str = "lpeg";

const RE_FILE: &str = "re.lua";

const RE_ENV: &str = "LPEG_RE_PATH";

pub fn compile(headers: &Path) -> Result<(), Box<dyn Error>> {
    let source = vendor::compile(NAME, headers)?;

    let re = source.join(RE_FILE);
    if !re.is_file() {
        return Err(format!("no {RE_FILE} in {}", source.display()).into());
    }

    println!("cargo::rustc-env={RE_ENV}={}", re.display());

    Ok(())
}
