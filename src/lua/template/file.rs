use std::path::Path;

use anyhow::{Context, Result};

use super::load::destination;
use crate::lua::Shared;
use crate::lua::constants::MODULES_DIR;
use crate::lua::embed;
use crate::lua::ld::{API, Paths, Surface, extend_module_path, install, share};
use crate::lua::runtime::{add_module_path, environment, runtime};
use crate::lua::scope::{Content, Output};
use crate::state::Classes;
use crate::utils;

pub fn load_template_file(
    command: &str,
    home: &Path,
    repo: &Path,
    path: &Path,
    classes: &Classes,
    config: &Shared,
) -> Result<Output> {
    let source = std::fs::read_to_string(path)
        .with_context(|| format!("{command}: failed to read {}", path.display()))?;
    let dirs = utils::config_dir()
        .with_context(|| format!("{command}: failed to locate the configuration"))?;
    let data = utils::data_dir()
        .with_context(|| format!("{command}: failed to locate the data directory"))?;
    let paths = Paths::new(home, &dirs, &data).with_repo(Some(repo));

    render(command, home, repo, &paths, path, &source, classes, config)
}

#[allow(clippy::too_many_arguments)]
fn render(
    command: &str,
    home: &Path,
    repo: &Path,
    paths: &Paths,
    path: &Path,
    source: &str,
    classes: &Classes,
    shared: &Shared,
) -> Result<Output> {
    let dest = destination(command, home, repo, path)?;
    let chunk = embed::compile(source)
        .with_context(|| format!("{command}: failed to compile {}", path.display()))?;

    let lua = runtime().with_context(|| format!("{command}: failed to start the Lua runtime"))?;
    let mut paths = paths.clone();
    if let Some(dir) = path.parent() {
        paths = paths.with_dir(dir);
    }
    install(&lua, Surface::Standalone, &paths, classes)
        .with_context(|| format!("{command}: failed to install `{API}`"))?;
    share(&lua, shared);
    extend_module_path(&lua)
        .with_context(|| format!("{command}: failed to reach the registered modules"))?;
    add_module_path(&lua, paths.config())
        .with_context(|| format!("{command}: failed to make {MODULES_DIR}/ requirable"))?;

    let rendered = environment(&lua, None)
        .and_then(|environment| embed::run(&lua, chunk, &path.display().to_string(), environment))
        .with_context(|| format!("{command}: failed to run {}", path.display()))?;

    Ok(Output::new(dest, Content::Text(rendered), None, None))
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use crate::lua::{Config, Shared};

    fn configuration() -> Shared {
        Arc::new(Mutex::new(Config::default()))
    }

    use std::path::Path;

    use super::*;

    fn load(root: &Path, name: &str, source: &str, classes: &Classes) -> Result<Output> {
        let home = root.join("home");
        let repo = root.join("repo");
        let path = repo.join(name);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(&path, source).unwrap();

        load_template_file("alt", &home, &repo, &path, classes, &configuration())
    }

    #[test]
    fn classes_are_reachable() {
        let root = tempfile::tempdir().unwrap();
        let mut classes = Classes::default();
        classes.set("form-factor", "laptop");

        let output = load(
            root.path(),
            ".zshrc.luadot",
            "on a <%= ld.class.get(\"form-factor\") %>",
            &classes,
        )
        .unwrap();

        assert_eq!(output.content(), &Content::Text("on a laptop".to_string()));
    }

    #[test]
    fn alternatives_resolve_beside_the_file() {
        let root = tempfile::tempdir().unwrap();
        let beside = root.path().join("repo");
        std::fs::create_dir_all(&beside).unwrap();
        std::fs::write(beside.join("aliases.zsh"), "alias ll='ls -l'").unwrap();

        let output = load(
            root.path(),
            ".zshrc.luadot",
            "<%= ld.alt.read(\"aliases.zsh\") %> in <%= ld.path.dir %>",
            &Classes::default(),
        )
        .unwrap();

        assert_eq!(
            output.content(),
            &Content::Text(format!("alias ll='ls -l' in {}", beside.display()))
        );
    }
}
