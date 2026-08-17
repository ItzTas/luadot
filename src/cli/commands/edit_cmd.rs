use std::ffi::OsStr;
use std::path::Path;

use anyhow::{Context, Result};
use clap::Args;

use crate::crypt;
use crate::lua::Config;
use crate::output;
use crate::utils::{self, Workspace};

#[derive(Debug, Args)]
pub struct EditArgs {
    #[arg(value_name = "PATH")]
    pub path: String,
}

pub fn edit_cmd(args: EditArgs) -> Result<()> {
    let Workspace { config, home, repo } = utils::workspace("edit")?;
    let in_repo = utils::managed_path("edit", &home, &repo, &args.path)?;

    let Some((stripped, backend)) = crypt::split(utils::relative(&repo, &in_repo)) else {
        return utils::open("edit", &in_repo);
    };

    edit_encrypted(&config, backend, &home, &in_repo, &stripped)
}

fn edit_encrypted(
    config: &Config,
    backend: crypt::Backend,
    home: &Path,
    in_repo: &Path,
    stripped: &Path,
) -> Result<()> {
    let identity = config
        .crypt_identity()
        .map(|path| utils::expand(home, path));
    let name = stripped.file_name().unwrap_or(OsStr::new("plaintext"));

    let workspace = crypt::Workspace::create("edit")?;
    let plain = workspace.file(name);
    crypt::decrypt_into("edit", backend, identity.as_deref(), in_repo, &plain)?;
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

    reencrypt(config, backend, &plain, in_repo)
}

fn reencrypt(config: &Config, backend: crypt::Backend, plain: &Path, in_repo: &Path) -> Result<()> {
    let mut staging = in_repo.as_os_str().to_os_string();
    staging.push(".tmp");
    let staging = Path::new(&staging);

    let placed = crypt::encrypt("edit", backend, config.crypt_recipients(), plain, staging)
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
