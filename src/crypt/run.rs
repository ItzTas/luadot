use std::io::Write;
use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};

use anyhow::{Context, Result, bail};

use super::backend::Backend;
use super::constants::{GPG_FLAGS, GPG_PASSPHRASE_FLAGS};
use super::lock::Lock;
use super::plugin;

pub fn encrypt(
    command: &str,
    backend: Backend,
    lock: Lock,
    recipients: &[String],
    source: &Path,
    dest: &Path,
) -> Result<()> {
    require_recipients(command, lock, recipients)?;
    plugin::for_recipients(command, backend, lock, recipients)?;
    run(
        command,
        encrypt_command(backend, lock, recipients, Some(source), dest),
    )
    .map(|_| ())
}

pub fn encrypt_contents(
    command: &str,
    backend: Backend,
    lock: Lock,
    recipients: &[String],
    contents: &[u8],
    dest: &Path,
) -> Result<()> {
    require_recipients(command, lock, recipients)?;
    plugin::for_recipients(command, backend, lock, recipients)?;
    piped(
        command,
        encrypt_command(backend, lock, recipients, None, dest),
        contents,
    )
}

pub fn decrypt(
    command: &str,
    backend: Backend,
    lock: Lock,
    identity: Option<&Path>,
    source: &Path,
) -> Result<Vec<u8>> {
    require_identity(command, backend, lock, identity)?;
    plugin::for_identity(command, backend, lock, identity)?;
    run(
        command,
        decrypt_command(backend, lock, identity, source, None),
    )
}

pub fn decrypt_into(
    command: &str,
    backend: Backend,
    lock: Lock,
    identity: Option<&Path>,
    source: &Path,
    dest: &Path,
) -> Result<()> {
    require_identity(command, backend, lock, identity)?;
    plugin::for_identity(command, backend, lock, identity)?;
    run(
        command,
        decrypt_command(backend, lock, identity, source, Some(dest)),
    )
    .map(|_| ())
}

pub fn require_recipients(command: &str, lock: Lock, recipients: &[String]) -> Result<()> {
    if lock.passphrase() {
        return Ok(());
    }
    if recipients.is_empty() {
        bail!(
            "{command}: no recipients set; call `ld.crypt.lock` with `recipients` in the configuration"
        );
    }
    Ok(())
}

fn require_identity(
    command: &str,
    backend: Backend,
    lock: Lock,
    identity: Option<&Path>,
) -> Result<()> {
    if lock.passphrase() {
        return Ok(());
    }
    if backend == Backend::Age && identity.is_none() {
        bail!(
            "{command}: decrypting with age needs `ld.crypt.lock` with `identity` in the configuration"
        );
    }
    Ok(())
}

fn encrypt_command(
    backend: Backend,
    lock: Lock,
    recipients: &[String],
    source: Option<&Path>,
    dest: &Path,
) -> Command {
    let mut invocation = Command::new(backend.name());
    match (backend, lock) {
        (Backend::Age, Lock::Keys) => {
            invocation.arg("--encrypt");
            to(&mut invocation, recipients);
        }
        (Backend::Age, Lock::Passphrase) => {
            invocation.args(["--encrypt", "--passphrase"]);
        }
        (Backend::Gpg, Lock::Keys) => {
            invocation.args(GPG_FLAGS).arg("--encrypt");
            to(&mut invocation, recipients);
        }
        (Backend::Gpg, Lock::Passphrase) => {
            invocation.args(GPG_PASSPHRASE_FLAGS).arg("--symmetric");
        }
    }
    invocation.arg("--output").arg(dest);
    if let Some(source) = source {
        invocation.arg(source);
    }
    invocation
}

fn to(invocation: &mut Command, recipients: &[String]) {
    for recipient in recipients {
        invocation.arg("--recipient").arg(recipient);
    }
}

fn decrypt_command(
    backend: Backend,
    lock: Lock,
    identity: Option<&Path>,
    source: &Path,
    dest: Option<&Path>,
) -> Command {
    let mut invocation = Command::new(backend.name());
    match (backend, lock) {
        (Backend::Age, Lock::Keys) => {
            invocation.arg("--decrypt");
            if let Some(identity) = identity {
                invocation.arg("--identity").arg(identity);
            }
        }
        (Backend::Age, Lock::Passphrase) => {
            invocation.arg("--decrypt");
        }
        (Backend::Gpg, Lock::Keys) => {
            invocation.args(GPG_FLAGS).arg("--decrypt");
        }
        (Backend::Gpg, Lock::Passphrase) => {
            invocation.args(GPG_PASSPHRASE_FLAGS).arg("--decrypt");
        }
    }
    if let Some(dest) = dest {
        invocation.arg("--output").arg(dest);
    }
    invocation.arg(source);
    invocation
}

fn run(command: &str, mut invocation: Command) -> Result<Vec<u8>> {
    let tool = invocation.get_program().to_string_lossy().into_owned();
    let output = invocation
        .output()
        .with_context(|| format!("{command}: failed to run `{tool}`; is it installed?"))?;

    reported(command, &tool, &output.status, &output.stderr)?;

    Ok(output.stdout)
}

fn piped(command: &str, mut invocation: Command, contents: &[u8]) -> Result<()> {
    let tool = invocation.get_program().to_string_lossy().into_owned();
    let mut child = invocation
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("{command}: failed to run `{tool}`; is it installed?"))?;

    let handed = || format!("{command}: failed to hand the plaintext to `{tool}`");
    let mut stdin = child.stdin.take().with_context(handed)?;
    let written = stdin.write_all(contents);
    drop(stdin);

    let output = child
        .wait_with_output()
        .with_context(|| format!("{command}: failed to run `{tool}`"))?;

    reported(command, &tool, &output.status, &output.stderr)?;

    written.with_context(handed)
}

fn reported(command: &str, tool: &str, status: &ExitStatus, stderr: &[u8]) -> Result<()> {
    if status.success() {
        return Ok(());
    }

    bail!(
        "{command}: `{tool}` failed: {}",
        String::from_utf8_lossy(stderr).trim()
    )
}

#[cfg(test)]
mod tests {
    use std::ffi::OsStr;

    use super::*;

    fn args(command: &Command) -> Vec<String> {
        command
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect()
    }

    #[test]
    fn age_encrypts_to_every_recipient() {
        let recipients = ["age1first".to_string(), "age1second".to_string()];
        let invocation = encrypt_command(
            Backend::Age,
            Lock::Keys,
            &recipients,
            Some(Path::new("/home/u/.netrc")),
            Path::new("/repo/.netrc.age"),
        );

        assert_eq!(invocation.get_program(), OsStr::new("age"));
        assert_eq!(
            args(&invocation),
            [
                "--encrypt",
                "--recipient",
                "age1first",
                "--recipient",
                "age1second",
                "--output",
                "/repo/.netrc.age",
                "/home/u/.netrc",
            ]
        );
    }

    #[test]
    fn age_decrypts_with_the_identity() {
        let invocation = decrypt_command(
            Backend::Age,
            Lock::Keys,
            Some(Path::new("/home/u/key.txt")),
            Path::new("/repo/.netrc.age"),
            None,
        );

        assert_eq!(
            args(&invocation),
            [
                "--decrypt",
                "--identity",
                "/home/u/key.txt",
                "/repo/.netrc.age",
            ]
        );
    }

    #[test]
    fn a_failing_tool_reports_its_stderr() {
        let mut invocation = Command::new("sh");
        invocation.args(["-c", "echo broken key >&2; exit 1"]);

        let err = run("apply", invocation).unwrap_err().to_string();

        assert_eq!(err, "apply: `sh` failed: broken key");
    }
}
