use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

const SHARED_WRITE: u32 = 0o022;
use crate::files::mode_bits;

pub fn trusted(path: &Path) -> Result<()> {
    let Some(real) = real_path(path)? else {
        return Ok(());
    };

    let meta = std::fs::symlink_metadata(&real)
        .with_context(|| format!("config: failed to inspect {}", real.display()))?;
    if mode_bits(&meta) & SHARED_WRITE != 0 {
        bail!(
            "config: {} is writable by group or others; take those bits off before luadot runs it",
            real.display()
        );
    }

    let owned = gix::sec::identity::is_path_owned_by_current_user(&real)
        .with_context(|| format!("config: failed to read the owner of {}", real.display()))?;
    if !owned {
        bail!(
            "config: {} belongs to another user; luadot will not run it",
            real.display()
        );
    }

    Ok(())
}

fn real_path(path: &Path) -> Result<Option<PathBuf>> {
    match std::fs::canonicalize(path) {
        Ok(real) => Ok(Some(real)),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(err) => {
            Err(err).with_context(|| format!("config: failed to inspect {}", path.display()))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs::Permissions;
    use std::os::unix::fs::PermissionsExt;

    use super::*;

    #[test]
    fn a_configuration_others_can_write_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.lua");
        std::fs::write(&path, "ld.opt.link(\"symbolic\")").unwrap();
        std::fs::set_permissions(&path, Permissions::from_mode(0o666)).unwrap();

        let err = trusted(&path).unwrap_err().to_string();
        assert!(err.contains("is writable by group or others"), "{err}");

        std::fs::set_permissions(&path, Permissions::from_mode(0o644)).unwrap();
        trusted(&path).unwrap();
    }

    #[test]
    fn a_missing_configuration_is_no_trouble() {
        let dir = tempfile::tempdir().unwrap();

        trusted(&dir.path().join("config.lua")).unwrap();
    }
}
