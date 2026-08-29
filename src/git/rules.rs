use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::utils::{data_dir, home_dir, managed_relative};

pub fn dir(command: &str, repo: &Path) -> Result<PathBuf> {
    let relative = managed_relative(&home_dir()?, &data_dir()?.join("git")).with_context(|| {
        format!("{command}: the data directory cannot hold the rules git reads")
    })?;

    Ok(repo.join(relative))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rules_live_under_the_data_mirror() {
        let repo = Path::new("/repo");
        let home = home_dir().unwrap();
        let data = data_dir().unwrap();

        assert_eq!(
            dir("git", repo).unwrap(),
            repo.join(data.strip_prefix(&home).unwrap()).join("git")
        );
    }
}
