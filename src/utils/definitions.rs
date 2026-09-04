use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::lua::{self, Placed};
use crate::output::{self, Tone};

use super::paths::{config_dir, data_dir, home_dir};

pub fn place_definitions(command: &str, dir: &Path, registered: &[PathBuf]) -> Result<()> {
    let home = home_dir()?;
    let data = data_dir()?;

    report(command, &lua::install_definitions(command, &data)?);
    report(
        command,
        &lua::point_at_definitions(command, dir, &home, &data, registered)?,
    );

    Ok(())
}

pub fn refresh_definitions() -> Result<()> {
    lua::refresh_definitions("meta", &data_dir()?)
}

pub fn offer_definitions(command: &str, registered: &[PathBuf]) {
    let placed = config_dir().and_then(|config| place_definitions(command, &config, registered));
    if let Err(err) = placed {
        output::warn(format!("{err:#}"));
    }
}

fn report(command: &str, placed: &Placed) {
    match placed {
        Placed::Written(path) => output::entry(Tone::Good, "wrote", path.display()),
        Placed::Merged(path) => output::entry(Tone::Good, "merged", path.display()),
        Placed::Kept(path, wanted) => {
            output::warn(format!(
                "{command}: {} could not be parsed and was left alone; add this to it:",
                path.display()
            ));
            output::line(wanted);
        }
    }
}
