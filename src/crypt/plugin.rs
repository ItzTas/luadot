use std::borrow::Borrow;
use std::collections::BTreeSet;
use std::env;
use std::ffi::OsStr;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use anyhow::{Result, bail};

use super::backend::Backend;
const EXECUTABLE: u32 = 0o111;
use super::lock::Lock;

static CHECKED: Mutex<BTreeSet<String>> = Mutex::new(BTreeSet::new());

static CLEARED: Mutex<BTreeSet<PathBuf>> = Mutex::new(BTreeSet::new());

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
    if seen(&CLEARED, identity) {
        return Ok(());
    }
    let Ok(contents) = std::fs::read_to_string(identity) else {
        return Ok(());
    };

    let what = format!("the identity {}", identity.display());
    for name in from_identity(&contents) {
        require(command, &name, &what)?;
    }
    remember(&CLEARED, identity.to_path_buf());

    Ok(())
}

pub fn plugged(identity: Option<&Path>) -> bool {
    let Some(identity) = identity else {
        return false;
    };
    let Ok(contents) = std::fs::read_to_string(identity) else {
        return false;
    };

    !from_identity(&contents).is_empty()
}

fn require(command: &str, name: &str, what: &str) -> Result<()> {
    if seen(&CHECKED, name) {
        return Ok(());
    }

    let binary = binary(name);
    if in_path(&binary, env::var_os("PATH").as_deref()).is_none() {
        bail!("{command}: {what} needs `{binary}`, which is not on your PATH");
    }
    remember(&CHECKED, name.to_string());

    Ok(())
}

fn seen<T, Q>(memo: &Mutex<BTreeSet<T>>, key: &Q) -> bool
where
    T: Ord + Borrow<Q>,
    Q: Ord + ?Sized,
{
    memo.lock().is_ok_and(|memo| memo.contains(key))
}

fn remember<T: Ord>(memo: &Mutex<BTreeSet<T>>, key: T) {
    if let Ok(mut memo) = memo.lock() {
        memo.insert(key);
    }
}

fn binary(name: &str) -> String {
    format!("age-plugin-{name}")
}

fn from_recipient(recipient: &str) -> Option<String> {
    let lowered = recipient.to_lowercase();
    let name = lowered.rsplit_once('1')?.0.strip_prefix("age1")?;
    match name.is_empty() {
        true => None,
        false => Some(name.to_string()),
    }
}

fn from_identity(contents: &str) -> Vec<String> {
    let mut names = Vec::new();
    for line in contents.lines().map(str::trim) {
        let Some(name) = named(line) else {
            continue;
        };
        if !names.contains(&name) {
            names.push(name);
        }
    }
    names
}

fn named(line: &str) -> Option<String> {
    let name = line
        .strip_prefix("AGE-PLUGIN-")?
        .rsplit_once('-')?
        .0
        .to_lowercase();

    match name.is_empty() {
        true => None,
        false => Some(name),
    }
}

fn in_path(program: &str, path: Option<&OsStr>) -> Option<PathBuf> {
    env::split_paths(path?)
        .map(|dir| dir.join(program))
        .find(|candidate| runnable(candidate))
}

fn runnable(candidate: &Path) -> bool {
    let Ok(metadata) = candidate.metadata() else {
        return false;
    };

    metadata.is_file() && metadata.permissions().mode() & EXECUTABLE != 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_every_plugin_once() {
        let contents = concat!(
            "AGE-PLUGIN--1QQQPQ\n",
            "AGE-SECRET-KEY-1QQQPQ\n",
            "AGE-PLUGIN-YUBIKEY-1QQQPQ\n",
            "AGE-PLUGIN-TPM-1QQQPQ\n",
            "AGE-PLUGIN-YUBIKEY-1QQQPZ\n",
        );

        assert_eq!(from_identity(contents), ["yubikey", "tpm"]);
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
}
