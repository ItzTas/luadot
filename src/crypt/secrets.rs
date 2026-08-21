use std::path::Path;

use super::identity::{Identity, Key};
use super::lock::{Lock, lock};
use crate::utils;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Secrets {
    Keys {
        recipients: Vec<String>,
        identity: Option<Key>,
    },
    Passphrase,
}

impl Default for Secrets {
    fn default() -> Self {
        Self::Keys {
            recipients: Vec::new(),
            identity: None,
        }
    }
}

impl Secrets {
    pub fn lock(&self, warn: bool) -> Lock {
        lock(matches!(self, Self::Passphrase), warn)
    }

    pub fn recipients(&self) -> &[String] {
        match self {
            Self::Keys { recipients, .. } => recipients,
            Self::Passphrase => &[],
        }
    }

    pub fn identity(&self, home: &Path) -> Identity {
        let Self::Keys { identity, .. } = self else {
            return Identity::default();
        };

        Identity::new(identity.as_ref().map(|key| resolved(home, key)))
    }
}

fn resolved(home: &Path, key: &Key) -> Key {
    match key {
        Key::File(path) => Key::File(utils::expand(home, path)),
        Key::Command(provider) => Key::Command(provider.clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn the_identity_path_is_resolved_against_home() {
        let secrets = Secrets::Keys {
            recipients: Vec::new(),
            identity: Some(Key::File(PathBuf::from("~/.keys/age.txt"))),
        };

        let mut identity = secrets.identity(Path::new("/home/u"));

        assert_eq!(
            identity.path("apply").unwrap(),
            Some(Path::new("/home/u/.keys/age.txt"))
        );
    }
}
