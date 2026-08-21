use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::Args;

use crate::crypt;
use crate::lua::Config;
use crate::output::{self, Tone};
use crate::utils::{self, Workspace};

#[derive(Debug, Args)]
pub struct RekeyArgs {
    #[arg(value_name = "PATH", help = "Narrow the run to this file or directory")]
    pub path: Option<String>,
    #[arg(
        short = 'n',
        long,
        help = "Report what would be re-encrypted, touching nothing"
    )]
    pub dry_run: bool,
}

#[derive(Debug, PartialEq, Eq)]
struct Secret {
    file: PathBuf,
    stripped: PathBuf,
    backend: crypt::Backend,
}

pub fn rekey_cmd(args: RekeyArgs) -> Result<()> {
    let Workspace { config, home, repo } = utils::workspace("rekey")?;
    let config = utils::configured("rekey", &config)?;

    let root = utils::managed_root("rekey", &home, &repo, args.path.as_deref())?;
    let files = utils::managed_files("rekey", &repo, &root, |relative| {
        config.is_ignored(&crypt::logical(relative))
    })?;

    let secrets = secrets(&repo, &files);
    if secrets.is_empty() {
        output::note("nothing to re-encrypt");
        return Ok(());
    }

    let lock = config.crypt_lock();
    let backend = config.crypt_backend();
    crypt::require_recipients("rekey", lock, config.crypt_secrets().recipients())?;
    crypt::require_recipient_plugins("rekey", backend, lock, config.crypt_secrets().recipients())?;

    if args.dry_run {
        return foresee(&repo, &secrets, backend);
    }

    let mut identity = config.crypt_identity(&home);
    if secrets
        .iter()
        .any(|secret| secret.backend == crypt::Backend::Age)
    {
        crypt::require_identity_plugins(
            "rekey",
            crypt::Backend::Age,
            lock,
            identity.path("rekey")?,
        )?;
    }

    let mut ahead = crypt::Ahead::new(
        "rekey",
        lock,
        identity,
        secrets
            .iter()
            .map(|secret| (secret.backend, secret.file.clone()))
            .collect(),
    );

    for secret in &secrets {
        let target = target(&repo, secret, backend);
        rekey(&config, lock, &mut ahead, secret, &target)?;
        output::entry(
            Tone::Good,
            "rekeyed",
            utils::relative(&repo, &target).display(),
        );
    }

    output::note(format!(
        "re-encrypted {} secret(s) with {}; commit the repository to keep them",
        secrets.len(),
        backend.name()
    ));

    Ok(())
}

fn secrets(repo: &Path, files: &[PathBuf]) -> Vec<Secret> {
    files
        .iter()
        .filter_map(|file| {
            let (stripped, backend) = crypt::split(utils::relative(repo, file))?;
            Some(Secret {
                file: file.clone(),
                stripped,
                backend,
            })
        })
        .collect()
}

fn target(repo: &Path, secret: &Secret, backend: crypt::Backend) -> PathBuf {
    crypt::stored(&repo.join(&secret.stripped), backend)
}

fn foresee(repo: &Path, secrets: &[Secret], backend: crypt::Backend) -> Result<()> {
    for secret in secrets {
        let target = target(repo, secret, backend);
        let label = match target == secret.file {
            true => "re-encrypt",
            false => "re-encrypt as",
        };
        output::entry(Tone::Muted, label, utils::relative(repo, &target).display());
    }

    output::note(format!(
        "would re-encrypt {} secret(s) with {}",
        secrets.len(),
        backend.name()
    ));

    Ok(())
}

fn rekey(
    config: &Config,
    lock: crypt::Lock,
    ahead: &mut crypt::Ahead,
    secret: &Secret,
    target: &Path,
) -> Result<()> {
    let contents = ahead
        .take(secret.backend, &secret.file)
        .with_context(|| format!("rekey: failed to decrypt {}", secret.file.display()))?;

    let mut staging = target.as_os_str().to_os_string();
    staging.push(".tmp");
    let staging = Path::new(&staging);

    let placed = crypt::encrypt_contents(
        "rekey",
        config.crypt_backend(),
        lock,
        config.crypt_secrets().recipients(),
        &contents,
        staging,
    )
    .and_then(|()| {
        std::fs::rename(staging, target)
            .with_context(|| format!("rekey: failed to write {}", target.display()))
    });
    if placed.is_err() {
        let _ = std::fs::remove_file(staging);
        return placed;
    }

    if target != secret.file {
        std::fs::remove_file(&secret.file)
            .with_context(|| format!("rekey: failed to remove {}", secret.file.display()))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repo() -> PathBuf {
        PathBuf::from("/repo")
    }

    #[test]
    fn only_the_encrypted_files_are_collected() {
        let repo = repo();
        let files = [
            repo.join(".bashrc"),
            repo.join(".netrc.age"),
            repo.join(".config/wireguard/wg0.conf.gpg"),
        ];

        let secrets = secrets(&repo, &files);

        assert_eq!(
            secrets,
            [
                Secret {
                    file: repo.join(".netrc.age"),
                    stripped: PathBuf::from(".netrc"),
                    backend: crypt::Backend::Age,
                },
                Secret {
                    file: repo.join(".config/wireguard/wg0.conf.gpg"),
                    stripped: PathBuf::from(".config/wireguard/wg0.conf"),
                    backend: crypt::Backend::Gpg,
                },
            ]
        );
    }

    #[test]
    fn a_secret_moves_to_the_extension_of_the_configured_backend() {
        let repo = repo();
        let secrets = secrets(&repo, &[repo.join(".netrc.age")]);

        assert_eq!(
            target(&repo, &secrets[0], crypt::Backend::Gpg),
            repo.join(".netrc.gpg")
        );
    }
}
