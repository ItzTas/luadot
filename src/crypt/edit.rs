use std::ffi::OsStr;
use std::os::unix::fs::DirBuilderExt;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};

#[derive(Debug)]
pub struct Workspace {
    dir: PathBuf,
}

impl Workspace {
    pub fn create(command: &str) -> Result<Self> {
        let dir = base_dir().join(unique_name());
        std::fs::DirBuilder::new()
            .mode(0o700)
            .create(&dir)
            .with_context(|| format!("{command}: failed to create {}", dir.display()))?;
        Ok(Self { dir })
    }

    pub fn file(&self, name: &OsStr) -> PathBuf {
        self.dir.join(name)
    }

    pub fn remove(&self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

impl Drop for Workspace {
    fn drop(&mut self) {
        self.remove();
    }
}

fn base_dir() -> PathBuf {
    std::env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .filter(|dir| dir.is_dir())
        .unwrap_or_else(std::env::temp_dir)
}

fn unique_name() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_nanos())
        .unwrap_or_default();
    format!("luadot-edit-{}-{nanos}", std::process::id())
}

#[cfg(test)]
mod tests {
    use std::os::unix::fs::PermissionsExt;

    use super::*;

    #[test]
    fn a_workspace_is_private_to_the_user() {
        let workspace = Workspace::create("edit").unwrap();

        let mode = std::fs::metadata(&workspace.dir)
            .unwrap()
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(mode, 0o700);
    }
}
