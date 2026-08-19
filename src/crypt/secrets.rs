use std::path::{Path, PathBuf};

use super::identity::{Identity, Provider};
use super::lock::{Lock, lock};
use crate::utils;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Secrets {
    Keys {
        recipients: Vec<String>,
        identity: Option<PathBuf>,
        identity_command: Option<Provider>,
    },
    Passphrase,
}

impl Default for Secrets {
    fn default() -> Self {
        Self::Keys {
            recipients: Vec::new(),
            identity: None,
            identity_command: None,
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
        let Self::Keys {
            identity,
            identity_command,
            ..
        } = self
        else {
            return Identity::default();
        };

        Identity::new(
            identity.as_deref().map(|path| utils::expand(home, path)),
            identity_command.clone(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
            identity_command: None,
        };

        assert_eq!(secrets.recipients(), ["age1example"]);
    }

    #[test]
    fn the_identity_path_is_resolved_against_home() {
        let secrets = Secrets::Keys {
            recipients: Vec::new(),
            identity: Some(PathBuf::from("~/.keys/age.txt")),
            identity_command: None,
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
