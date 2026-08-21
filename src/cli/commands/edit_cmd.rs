use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::Args;

use crate::crypt;
use crate::files;
use crate::lua::{Config, TEMPLATE_FILE};
use crate::output;
use crate::utils::{self, Workspace};

#[derive(Debug, Args)]
pub struct EditArgs {
    #[arg(value_name = "PATH")]
    pub path: String,
}

pub fn edit_cmd(args: EditArgs) -> Result<()> {
    let Workspace { config, home, repo } = utils::workspace("edit")?;
    let config = utils::configured("edit", &config)?;
    let in_repo = utils::managed_path("edit", &home, &repo, &args.path)?;

    if let Some(script) = template_script(&in_repo) {
        return utils::open("edit", &script);
    }

    let Some((stripped, backend)) = crypt::split(utils::relative(&repo, &in_repo)) else {
        return utils::open("edit", &in_repo);
    };

    edit_encrypted(&config, backend, &home, &in_repo, &stripped)
}

fn template_script(in_repo: &Path) -> Option<PathBuf> {
    if !files::is_template(in_repo) || !in_repo.is_dir() {
        return None;
    }

    Some(in_repo.join(TEMPLATE_FILE))
}

fn edit_encrypted(
    config: &Config,
    backend: crypt::Backend,
    home: &Path,
    in_repo: &Path,
    stripped: &Path,
) -> Result<()> {
    let lock = config.crypt_lock();
    let mut identity = config.crypt_identity(home);
    let name = stripped.file_name().unwrap_or(OsStr::new("plaintext"));

    crypt::require_identity_plugins("edit", backend, lock, identity.path("edit")?)?;
    crypt::require_recipient_plugins("edit", backend, lock, config.crypt_secrets().recipients())?;

    let workspace = crypt::Workspace::create("edit")?;
    let plain = workspace.file(name);
    crypt::decrypt_into(
        "edit",
        backend,
        lock,
        identity.path("edit")?,
        in_repo,
        &plain,
    )?;
    let before = read(&plain)?;

    let status = utils::launch("edit", &plain)?;
    if !status.success() {
        workspace.remove();
        std::process::exit(status.code().unwrap_or(1));
    }

    if read(&plain)? == before {
        output::note(format!("{} is unchanged", stripped.display()));
        return Ok(());
    }

    reencrypt(config, backend, lock, &plain, in_repo)
}

fn reencrypt(
    config: &Config,
    backend: crypt::Backend,
    lock: crypt::Lock,
    plain: &Path,
    in_repo: &Path,
) -> Result<()> {
    let mut staging = in_repo.as_os_str().to_os_string();
    staging.push(".tmp");
    let staging = Path::new(&staging);

    let placed = crypt::encrypt(
        "edit",
        backend,
        lock,
        config.crypt_secrets().recipients(),
        plain,
        staging,
    )
    .and_then(|()| {
        std::fs::rename(staging, in_repo)
            .with_context(|| format!("edit: failed to write {}", in_repo.display()))
    });
    if placed.is_err() {
        let _ = std::fs::remove_file(staging);
    }
    placed
}

fn read(plain: &Path) -> Result<Vec<u8>> {
    std::fs::read(plain).with_context(|| format!("edit: failed to read {}", plain.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_template_directory_is_edited_through_its_script() {
        let repo = tempfile::tempdir().unwrap();
        let template = repo.path().join("home/.zshrc.luadot");
        std::fs::create_dir_all(&template).unwrap();

        assert_eq!(
            template_script(&template),
            Some(template.join(TEMPLATE_FILE))
        );
    }

    #[test]
    fn a_standalone_template_is_edited_as_it_stands() {
        let repo = tempfile::tempdir().unwrap();
        let template = repo.path().join("home/.zprofile.luadot");
        std::fs::create_dir_all(template.parent().unwrap()).unwrap();
        std::fs::write(&template, "export HOST=1\n").unwrap();

        assert_eq!(template_script(&template), None);
    }

    #[test]
    fn a_managed_file_is_edited_as_it_stands() {
        let repo = tempfile::tempdir().unwrap();
        let file = repo.path().join("home/.vimrc");
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(&file, "set number\n").unwrap();

        assert_eq!(template_script(&file), None);
    }
}
