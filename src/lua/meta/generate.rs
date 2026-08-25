use anyhow::{Context, Result, bail};

use super::constants::{JSON_FAILED, JSON_FLAG, USAGE};
use super::render::render;
use crate::lua::ld::walker;

pub fn generate(args: impl Iterator<Item = String>) -> Result<String> {
    let args: Vec<String> = args.collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();

    match args.as_slice() {
        [] => Ok(render(&walker())),
        [flag] if *flag == JSON_FLAG => walker().to_json_pretty().context(JSON_FAILED),
        _ => bail!("{USAGE}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_json_carries_the_description_for_the_documentation_generator() {
        let json = generate([JSON_FLAG.to_string()].into_iter()).unwrap();
        let walker: tealr::TypeWalker = serde_json::from_str(&json).unwrap();

        assert!(walker.check_correct_version());
        assert!(!walker.given_types.is_empty());
    }
}
