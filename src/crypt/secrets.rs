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
    fn the_default_locks_with_keys_and_carries_none() {
        let secrets = Secrets::default();

        assert_eq!(secrets.lock(false), Lock::Keys);
        assert!(secrets.recipients().is_empty());
    }

    #[test]
    fn the_passphrase_form_carries_no_recipients_of_its_own() {
        let secrets = Secrets::Passphrase;

        assert_eq!(secrets.lock(false), Lock::Passphrase);
        assert!(secrets.recipients().is_empty());
    }

    #[test]
    fn the_key_form_hands_back_what_it_was_given() {
        let secrets = Secrets::Keys {
            recipients: vec!["age1example".to_string()],
            identity: None,
        };

        assert_eq!(secrets.recipients(), ["age1example"]);
    }

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

    #[test]
    fn the_passphrase_form_has_no_identity() {
        let mut identity = Secrets::Passphrase.identity(Path::new("/home/u"));

        assert_eq!(identity.path("apply").unwrap(), None);
    }
}
