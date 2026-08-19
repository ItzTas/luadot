use std::path::Path;

use super::parse::external;

pub fn require<'a>(repo: Option<&'a Path>, command: &str) -> mlua::Result<&'a Path> {
    repo.ok_or_else(|| {
        external(format!(
            "{command}: no repository set; run `luadot clone <url>` or `luadot init` first"
        ))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn yields_the_repository_when_set() {
        let repo = Path::new("/data/repo");

        assert_eq!(require(Some(repo), "`ld.setup`").unwrap(), repo);
    }

    #[test]
    fn reports_a_missing_repository() {
        let err = require(None, "`ld.setup`").unwrap_err().to_string();

        assert!(err.contains("`ld.setup`: no repository set"));
    }
}
