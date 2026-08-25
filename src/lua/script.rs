use std::path::Path;
use std::sync::MutexGuard;

use anyhow::{Context, Result, anyhow};

use super::constants::MODULES_DIR;
use super::ld::{API, Paths, Surface, extend_module_path, install, share};
use super::runtime::{add_module_path, runtime};
use super::{Config, Shared};
use crate::state::Classes;

pub fn run_script(
    command: &str,
    surface: Surface,
    path: &Path,
    modules: &[&Path],
    paths: &Paths,
    classes: &Classes,
    config: &Shared,
) -> Result<()> {
    let source = std::fs::read_to_string(path)
        .with_context(|| format!("{command}: failed to read {}", path.display()))?;

    run_source(
        command,
        surface,
        &source,
        &path.display().to_string(),
        modules,
        paths,
        classes,
        config,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn run_source(
    command: &str,
    surface: Surface,
    source: &str,
    name: &str,
    modules: &[&Path],
    paths: &Paths,
    classes: &Classes,
    config: &Shared,
) -> Result<()> {
    let lua = runtime().with_context(|| format!("{command}: failed to start the Lua runtime"))?;
    install(&lua, surface, paths, classes)
        .with_context(|| format!("{command}: failed to install `{API}`"))?;
    share(&lua, config);
    extend_module_path(&lua)
        .with_context(|| format!("{command}: failed to reach the registered modules"))?;

    for dir in modules.iter().rev() {
        add_module_path(&lua, dir)
            .with_context(|| format!("{command}: failed to make {MODULES_DIR}/ requirable"))?;
    }

    lua.load(source)
        .set_name(name)
        .exec()
        .with_context(|| format!("{command}: failed to run {name}"))?;

    lua.remove_app_data::<Shared>();
    kept(command, config)?.keep_runtime(lua);

    Ok(())
}

fn kept<'a>(command: &str, config: &'a Shared) -> Result<MutexGuard<'a, Config>> {
    config
        .lock()
        .map_err(|_| anyhow!("{command}: the configuration is still being changed"))
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use super::*;
    use crate::files::LinkMode;
    use crate::lua::from_source;
    use crate::lua::ld::plugin;

    #[test]
    fn a_registered_directory_is_requirable() {
        let root = tempfile::tempdir().unwrap();
        let dir = plugin(root.path(), "links", r#"ld.opt.link("symbolic")"#);
        let config = from_source(&format!(r#"ld.rtp.add("{}")"#, dir.display())).unwrap();
        let shared = Arc::new(Mutex::new(config));
        let paths = Paths::new(
            root.path(),
            &root.path().join(".config/luadot"),
            &root.path().join(".local/share/luadot"),
        );

        run_source(
            "setup",
            Surface::Setup,
            r#"require("links")"#,
            "setup.lua",
            &[],
            &paths,
            &Classes::default(),
            &shared,
        )
        .unwrap();

        assert_eq!(
            shared.lock().unwrap().link_mode(Path::new(".bashrc")),
            LinkMode::Symbolic
        );
    }
}
