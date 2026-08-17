use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, bail};

use super::backend::Backend;
use super::constants::GPG_FLAGS;

pub fn encrypt(
    command: &str,
    backend: Backend,
    recipients: &[String],
    source: &Path,
    dest: &Path,
) -> Result<()> {
    if recipients.is_empty() {
        bail!("{command}: no recipients set; call `ld.crypt.recipients` in the configuration");
    }
    run(command, encrypt_command(backend, recipients, source, dest)).map(|_| ())
}

pub fn decrypt(
    command: &str,
    backend: Backend,
    identity: Option<&Path>,
    source: &Path,
) -> Result<Vec<u8>> {
    require_identity(command, backend, identity)?;
    run(command, decrypt_command(backend, identity, source, None))
}

pub fn decrypt_into(
    command: &str,
    backend: Backend,
    identity: Option<&Path>,
    source: &Path,
    dest: &Path,
) -> Result<()> {
    require_identity(command, backend, identity)?;
    run(command, decrypt_command(backend, identity, source, Some(dest))).map(|_| ())
}

fn require_identity(command: &str, backend: Backend, identity: Option<&Path>) -> Result<()> {
    if backend == Backend::Age && identity.is_none() {
        bail!(
            "{command}: decrypting with age needs `ld.crypt.identity` in the configuration"
        );
    }
    Ok(())
}

fn encrypt_command(backend: Backend, recipients: &[String], source: &Path, dest: &Path) -> Command {
    let mut invocation = Command::new(backend.name());
    match backend {
        Backend::Age => {
            invocation.arg("--encrypt");
        }
        Backend::Gpg => {
            invocation.args(GPG_FLAGS).arg("--encrypt");
        }
    }
    for recipient in recipients {
        invocation.arg("--recipient").arg(recipient);
    }
    invocation.arg("--output").arg(dest).arg(source);
    invocation
}

fn decrypt_command(
    backend: Backend,
    identity: Option<&Path>,
    source: &Path,
    dest: Option<&Path>,
) -> Command {
    let mut invocation = Command::new(backend.name());
    match backend {
        Backend::Age => {
            invocation.arg("--decrypt");
            if let Some(identity) = identity {
                invocation.arg("--identity").arg(identity);
            }
        }
        Backend::Gpg => {
            invocation.args(GPG_FLAGS).arg("--decrypt");
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

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("{command}: `{tool}` failed: {}", stderr.trim());
    }

    Ok(output.stdout)
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
            &recipients,
            Path::new("/home/u/.netrc"),
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
            &recipients,
            Path::new("/home/u/.netrc"),
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
    fn age_decrypts_with_the_identity() {
        let invocation = decrypt_command(
            Backend::Age,
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
        let err = decrypt("apply", Backend::Age, None, Path::new("/repo/x.age"))
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
