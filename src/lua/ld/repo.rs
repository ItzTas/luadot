use std::path::Path;

use super::parse::external;

pub fn require<'a>(repo: Option<&'a Path>, command: &str) -> mlua::Result<&'a Path> {
    repo.ok_or_else(|| {
        external(format!(
            "{command}: no repository set; run `luadot clone <url>` or `luadot init` first"
        ))
    })
}
