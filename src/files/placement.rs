use std::fs::Permissions;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, bail};
use tracing::debug;

use super::constants::CHOWN;
use super::fs::mode_bits;
use super::link::LinkMode;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Attributes<'a> {
    mode: Option<u32>,
    owner: Option<&'a str>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Placement<'a> {
    link: LinkMode,
    attributes: Attributes<'a>,
}

impl<'a> Placement<'a> {
    pub fn new(link: LinkMode) -> Self {
        Self {
            link,
            attributes: Attributes::default(),
        }
    }

    pub fn with_link(self, link: LinkMode) -> Self {
        Self { link, ..self }
    }

    pub fn with_mode(self, mode: Option<u32>) -> Self {
        Self {
            attributes: self.attributes.with_mode(mode),
            ..self
        }
    }

    pub fn with_owner(self, owner: Option<&'a str>) -> Self {
        Self {
            attributes: self.attributes.with_owner(owner),
            ..self
        }
    }

    pub fn link(&self) -> LinkMode {
        self.link
    }

    pub fn attributes(&self) -> Attributes<'a> {
        self.attributes
    }

    pub fn mode(&self) -> Option<u32> {
        self.attributes.mode()
    }

    pub fn owner(&self) -> Option<&'a str> {
        self.attributes.owner()
    }

    pub fn carried_by(&self, dest: &Path) -> bool {
        self.attributes.carried_by(dest)
    }

    pub fn set_on(&self, command: &str, dest: &Path) -> Result<()> {
        self.attributes.set_on(command, dest)
    }

    pub fn own(&self, command: &str, dest: &Path) -> Result<()> {
        self.attributes.own(command, dest)
    }
}

impl<'a> Attributes<'a> {
    pub fn with_mode(self, mode: Option<u32>) -> Self {
        Self { mode, ..self }
    }

    pub fn with_owner(self, owner: Option<&'a str>) -> Self {
        Self { owner, ..self }
    }

    pub fn mode(&self) -> Option<u32> {
        self.mode
    }

    pub fn owner(&self) -> Option<&'a str> {
        self.owner
    }

    pub fn carried_by(&self, dest: &Path) -> bool {
        let Some(mode) = self.mode else {
            return true;
        };

        matches!(std::fs::metadata(dest), Ok(meta) if mode_bits(&meta) == mode)
    }

    pub fn set_on(&self, command: &str, dest: &Path) -> Result<()> {
        if let Some(mode) = self.mode {
            std::fs::set_permissions(dest, Permissions::from_mode(mode)).with_context(|| {
                format!("{command}: failed to set the mode of {}", dest.display())
            })?;
        }

        self.own(command, dest)
    }

    pub fn own(&self, command: &str, dest: &Path) -> Result<()> {
        let Some(owner) = self.owner else {
            return Ok(());
        };

        let mut invocation = Command::new(CHOWN);
        invocation.arg("--").arg(owner).arg(dest);
        debug!(?invocation, "setting the owner");

        let output = invocation
            .output()
            .with_context(|| format!("{command}: failed to run {CHOWN} for {}", dest.display()))?;
        if output.status.success() {
            return Ok(());
        }

        bail!(
            "{command}: {CHOWN} could not set the owner of {}: {}",
            dest.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        )
    }
}

#[cfg(test)]
mod tests {
    use std::os::unix::fs::MetadataExt;

    use super::*;

    fn bits(path: &Path) -> u32 {
        mode_bits(&std::fs::metadata(path).unwrap())
    }

    #[test]
    fn a_mode_is_compared_through_a_symlink() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("file");
        let link = dir.path().join("link");
        std::fs::write(&file, "x").unwrap();
        std::fs::set_permissions(&file, Permissions::from_mode(0o600)).unwrap();
        std::os::unix::fs::symlink(&file, &link).unwrap();

        let placement = Placement::default().with_mode(Some(0o600));
        assert!(placement.carried_by(&link));
        assert!(!placement.with_mode(Some(0o644)).carried_by(&link));
    }

    #[test]
    fn set_on_writes_mode_and_owner() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("file");
        std::fs::write(&file, "x").unwrap();
        let meta = std::fs::metadata(&file).unwrap();
        let owner = format!("{}:{}", meta.uid(), meta.gid());

        Placement::default()
            .with_mode(Some(0o640))
            .with_owner(Some(&owner))
            .set_on("apply", &file)
            .unwrap();

        assert_eq!(bits(&file), 0o640);
        assert_eq!(std::fs::metadata(&file).unwrap().uid(), meta.uid());
    }
}
