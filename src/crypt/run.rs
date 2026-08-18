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
        bail!("{command}: no recipients set; call `ld.crypt.recipients` in the configuration");
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
        bail!("{command}: decrypting with age needs `ld.crypt.identity` in the configuration");
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
            Path::new("/repo/home/.netrc.age"),
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
                "/repo/home/.netrc.age",
                "/home/u/.netrc",
            ]
        );
    }

    #[test]
    fn gpg_encrypts_without_prompting() {
        let recipients = ["me@example.com".to_string()];
        let invocation = encrypt_command(
            Backend::Gpg,
            Lock::Keys,
            &recipients,
            Some(Path::new("/home/u/.netrc")),
            Path::new("/repo/home/.netrc.gpg"),
        );

        assert_eq!(invocation.get_program(), OsStr::new("gpg"));
        assert_eq!(
            args(&invocation),
            [
                "--quiet",
                "--batch",
                "--yes",
                "--encrypt",
                "--recipient",
                "me@example.com",
                "--output",
                "/repo/home/.netrc.gpg",
                "/home/u/.netrc",
            ]
        );
    }

    #[test]
    fn a_plaintext_held_in_memory_is_encrypted_without_a_source_file() {
        let recipients = ["age1first".to_string()];
        let invocation = encrypt_command(
            Backend::Age,
            Lock::Keys,
            &recipients,
            None,
            Path::new("/repo/root/etc/wg0.conf.age"),
        );

        assert_eq!(
            args(&invocation),
            [
                "--encrypt",
                "--recipient",
                "age1first",
                "--output",
                "/repo/root/etc/wg0.conf.age",
            ]
        );
    }

    #[test]
    fn age_encrypts_to_a_passphrase_instead_of_recipients() {
        let recipients = ["age1first".to_string()];
        let invocation = encrypt_command(
            Backend::Age,
            Lock::Passphrase,
            &recipients,
            Some(Path::new("/home/u/.netrc")),
            Path::new("/repo/home/.netrc.age"),
        );

        assert_eq!(
            args(&invocation),
            [
                "--encrypt",
                "--passphrase",
                "--output",
                "/repo/home/.netrc.age",
                "/home/u/.netrc",
            ]
        );
    }

    #[test]
    fn gpg_encrypts_symmetrically_and_keeps_its_prompt() {
        let invocation = encrypt_command(
            Backend::Gpg,
            Lock::Passphrase,
            &[],
            Some(Path::new("/home/u/.netrc")),
            Path::new("/repo/home/.netrc.gpg"),
        );

        assert_eq!(
            args(&invocation),
            [
                "--quiet",
                "--yes",
                "--symmetric",
                "--output",
                "/repo/home/.netrc.gpg",
                "/home/u/.netrc",
            ]
        );
    }

    #[test]
    fn age_decrypts_a_passphrase_file_without_an_identity() {
        let invocation = decrypt_command(
            Backend::Age,
            Lock::Passphrase,
            Some(Path::new("/home/u/key.txt")),
            Path::new("/repo/home/.netrc.age"),
            None,
        );

        assert_eq!(args(&invocation), ["--decrypt", "/repo/home/.netrc.age"]);
    }

    #[test]
    fn gpg_decrypts_a_symmetric_file_with_its_prompt() {
        let invocation = decrypt_command(
            Backend::Gpg,
            Lock::Passphrase,
            None,
            Path::new("/repo/home/.netrc.gpg"),
            None,
        );

        assert_eq!(
            args(&invocation),
            ["--quiet", "--yes", "--decrypt", "/repo/home/.netrc.gpg"]
        );
    }

    #[test]
    fn a_passphrase_needs_neither_recipients_nor_an_identity() {
        assert!(require_recipients("add", Lock::Passphrase, &[]).is_ok());
        assert!(require_identity("apply", Backend::Age, Lock::Passphrase, None).is_ok());
    }

    #[test]
    fn what_is_piped_in_reaches_the_tool() {
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join("copied");
        let mut invocation = Command::new("sh");
        invocation.args(["-c", "cat > \"$0\"", dest.to_str().unwrap()]);

        piped("add", invocation, b"secret\n").unwrap();

        assert_eq!(std::fs::read(&dest).unwrap(), b"secret\n");
    }

    #[test]
    fn age_decrypts_with_the_identity() {
        let invocation = decrypt_command(
            Backend::Age,
            Lock::Keys,
            Some(Path::new("/home/u/key.txt")),
            Path::new("/repo/home/.netrc.age"),
            None,
        );

        assert_eq!(
            args(&invocation),
            [
                "--decrypt",
                "--identity",
                "/home/u/key.txt",
                "/repo/home/.netrc.age",
            ]
        );
    }

    #[test]
    fn gpg_decrypts_through_its_own_keyring() {
        let invocation = decrypt_command(
            Backend::Gpg,
            Lock::Keys,
            None,
            Path::new("/repo/home/.netrc.gpg"),
            Some(Path::new("/tmp/plain")),
        );

        assert_eq!(
            args(&invocation),
            [
                "--quiet",
                "--batch",
                "--yes",
                "--decrypt",
                "--output",
                "/tmp/plain",
                "/repo/home/.netrc.gpg",
            ]
        );
    }

    #[test]
    fn encrypting_without_recipients_is_refused() {
        let err = encrypt(
            "add",
            Backend::Age,
            Lock::Keys,
            &[],
            Path::new("/home/u/.netrc"),
            Path::new("/repo/home/.netrc.age"),
        )
        .unwrap_err()
        .to_string();

        assert!(err.contains("add: no recipients set"));
        assert!(err.contains("ld.crypt.recipients"));
    }

    #[test]
    fn decrypting_age_without_an_identity_is_refused() {
        let err = decrypt(
            "apply",
            Backend::Age,
            Lock::Keys,
            None,
            Path::new("/repo/x.age"),
        )
        .unwrap_err()
        .to_string();

        assert!(err.contains("apply: decrypting with age needs `ld.crypt.identity`"));
    }

    #[test]
    fn a_missing_tool_says_so() {
        let mut invocation = Command::new("luadot-tool-that-does-not-exist");
        invocation.arg("--version");

        let err = format!("{:#}", run("apply", invocation).unwrap_err());

        assert!(err.contains("apply: failed to run `luadot-tool-that-does-not-exist`"));
        assert!(err.contains("is it installed?"));
    }

    #[test]
    fn a_failing_tool_reports_its_stderr() {
        let mut invocation = Command::new("sh");
        invocation.args(["-c", "echo broken key >&2; exit 1"]);

        let err = run("apply", invocation).unwrap_err().to_string();

        assert_eq!(err, "apply: `sh` failed: broken key");
    }
}
