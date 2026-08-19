use std::env;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use anyhow::{Result, bail};

use super::backend::Backend;
use super::constants::{PLUGIN_BINARY, PLUGIN_IDENTITY, PLUGIN_RECIPIENT};
use super::lock::Lock;

pub fn for_recipients(
    command: &str,
    backend: Backend,
    lock: Lock,
    recipients: &[String],
) -> Result<()> {
    if backend != Backend::Age || lock.passphrase() {
        return Ok(());
    }

    for recipient in recipients {
        let Some(name) = from_recipient(recipient) else {
            continue;
        };
        require(command, &name, &format!("the recipient `{recipient}`"))?;
    }
    Ok(())
}

pub fn for_identity(
    command: &str,
    backend: Backend,
    lock: Lock,
    identity: Option<&Path>,
) -> Result<()> {
    if backend != Backend::Age || lock.passphrase() {
        return Ok(());
    }

    let Some(identity) = identity else {
        return Ok(());
    };
    let Ok(contents) = std::fs::read_to_string(identity) else {
        return Ok(());
    };
    let Some(name) = from_identity(&contents) else {
        return Ok(());
    };

    require(
        command,
        &name,
        &format!("the identity {}", identity.display()),
    )
}

fn require(command: &str, name: &str, what: &str) -> Result<()> {
    let binary = binary(name);
    if in_path(&binary, env::var_os("PATH").as_deref()).is_some() {
        return Ok(());
    }

    bail!("{command}: {what} needs `{binary}`, which is not on your PATH")
}

fn binary(name: &str) -> String {
    format!("{PLUGIN_BINARY}{name}")
}

fn from_recipient(recipient: &str) -> Option<String> {
    let name = recipient
        .rsplit_once('1')?
        .0
        .strip_prefix(PLUGIN_RECIPIENT)?;
    match name.is_empty() {
        true => None,
        false => Some(name.to_lowercase()),
    }
}

fn from_identity(contents: &str) -> Option<String> {
    contents
        .lines()
        .map(str::trim)
        .filter_map(|line| line.strip_prefix(PLUGIN_IDENTITY))
        .find_map(|rest| rest.rsplit_once('-'))
        .map(|(name, _)| name.to_lowercase())
        .filter(|name| !name.is_empty())
}

fn in_path(program: &str, path: Option<&OsStr>) -> Option<PathBuf> {
    env::split_paths(path?)
        .map(|dir| dir.join(program))
        .find(|candidate| candidate.is_file())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_plugin_recipient_names_its_binary() {
        assert_eq!(
            from_recipient(
                "age1yubikey1qwqvurupq2fzhaeg38g4hkyrfcyvpuhrjcnr6dtcxzftzmxtd8j7fqjhr5g"
            ),
            Some("yubikey".to_string())
        );
        assert_eq!(binary("yubikey"), "age-plugin-yubikey");
    }

    #[test]
    fn a_native_recipient_needs_no_plugin() {
        assert_eq!(
            from_recipient("age1ql3z7hjy54pw3hyww5ayyfg7zqgvc7w3j2elw8zmrj2kg5sfn9aqmcac8p"),
            None
        );
        assert_eq!(from_recipient("me@example.com"), None);
    }

    #[test]
    fn a_plugin_identity_names_its_binary() {
        assert_eq!(
            from_identity("AGE-PLUGIN-YUBIKEY-1QQQPQ8UEZQ\n"),
            Some("yubikey".to_string())
        );
        assert_eq!(
            from_identity("# created by age-plugin-tpm\nAGE-PLUGIN-TPM-EK-1QQQPQ\n"),
            Some("tpm-ek".to_string())
        );
    }

    #[test]
    fn a_native_identity_needs_no_plugin() {
        assert_eq!(
            from_identity("# public key: age1ql3z7\nAGE-SECRET-KEY-1QQQPQ\n"),
            None
        );
    }

    #[test]
    fn a_binary_is_looked_up_along_the_path() {
        let dir = tempfile::tempdir().unwrap();
        let binary = dir.path().join("age-plugin-luadot");
        std::fs::write(&binary, "#!/bin/sh\n").unwrap();
        let path = env::join_paths([dir.path()]).unwrap();

        assert_eq!(in_path("age-plugin-luadot", Some(&path)), Some(binary));
        assert_eq!(in_path("age-plugin-missing", Some(&path)), None);
        assert_eq!(in_path("age-plugin-luadot", None), None);
    }

    #[test]
    fn a_missing_plugin_stops_the_command() {
        let err = for_recipients(
            "add",
            Backend::Age,
            Lock::Keys,
            &["age1luadotplugin1qqqpq".to_string()],
        )
        .unwrap_err()
        .to_string();

        assert!(err.contains("add: the recipient `age1luadotplugin1qqqpq`"));
        assert!(err.contains("needs `age-plugin-luadotplugin`"));
    }

    #[test]
    fn gpg_and_passphrases_are_left_alone() {
        let recipients = ["age1luadotplugin1qqqpq".to_string()];

        assert!(for_recipients("add", Backend::Gpg, Lock::Keys, &recipients).is_ok());
        assert!(for_recipients("add", Backend::Age, Lock::Passphrase, &recipients).is_ok());
    }

    #[test]
    fn a_plugin_identity_file_stops_the_command() {
        let dir = tempfile::tempdir().unwrap();
        let identity = dir.path().join("plugin.txt");
        std::fs::write(&identity, "AGE-PLUGIN-LUADOTPLUGIN-1QQQPQ\n").unwrap();

        let err = for_identity("apply", Backend::Age, Lock::Keys, Some(&identity))
            .unwrap_err()
            .to_string();

        assert!(err.contains("needs `age-plugin-luadotplugin`"));
    }

    #[test]
    fn an_unreadable_or_missing_identity_is_left_to_age() {
        let dir = tempfile::tempdir().unwrap();

        assert!(for_identity("apply", Backend::Age, Lock::Keys, None).is_ok());
        assert!(
            for_identity(
                "apply",
                Backend::Age,
                Lock::Keys,
                Some(&dir.path().join("missing.txt")),
            )
            .is_ok()
        );
    }
}
