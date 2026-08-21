use std::path::Path;

use anyhow::{Context, Result};

use super::load::destination;
use crate::lua::constants::MODULES_DIR;
use crate::lua::embed;
use crate::lua::ld::{API, Paths, Surface, install, share};
use crate::lua::runtime::{add_module_path, environment, runtime};
use crate::lua::scope::{Content, Output};
use crate::lua::Shared;
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

    render(command, home, repo, &dirs, path, &source, classes, config)
}

#[allow(clippy::too_many_arguments)]
fn render(
    command: &str,
    home: &Path,
    repo: &Path,
    config: &Path,
    path: &Path,
    source: &str,
    classes: &Classes,
    shared: &Shared,
) -> Result<Output> {
    let dest = destination(command, home, repo, path)?;
    let chunk = embed::compile(source)
        .with_context(|| format!("{command}: failed to compile {}", path.display()))?;

    let lua = runtime().with_context(|| format!("{command}: failed to start the Lua runtime"))?;
    let mut paths = Paths::new(home, config).with_repo(Some(repo));
    if let Some(dir) = path.parent() {
        paths = paths.with_dir(dir);
    }
    install(&lua, Surface::Standalone, &paths, classes)
        .with_context(|| format!("{command}: failed to install `{API}`"))?;
    share(&lua, shared);
    add_module_path(&lua, config)
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
    fn the_machine_and_the_classes_are_reachable() {
        let root = tempfile::tempdir().unwrap();
        let mut classes = Classes::default();
        classes.set("form-factor", "laptop");

        let output = load(
            root.path(),
            "home/.zshrc.luadot",
            "<%= type(ld.sys.host.name) %> on a <%= ld.class.get(\"form-factor\") %>",
            &classes,
        )
        .unwrap();

        assert_eq!(
            output.content(),
            &Content::Text("string on a laptop".to_string())
        );
    }

    #[test]
    fn the_alternatives_resolve_next_to_the_file_itself() {
        let root = tempfile::tempdir().unwrap();
        let beside = root.path().join("repo/home");
        std::fs::create_dir_all(&beside).unwrap();
        std::fs::write(beside.join("aliases.zsh"), "alias ll='ls -l'").unwrap();

        let output = load(
            root.path(),
            "home/.zshrc.luadot",
            "<%= ld.alt.read(\"aliases.zsh\") %> in <%= ld.path.dir %>",
            &Classes::default(),
        )
        .unwrap();

        assert_eq!(
            output.content(),
            &Content::Text(format!("alias ll='ls -l' in {}", beside.display()))
        );
    }

    #[test]
    fn modules_of_the_configuration_are_requirable() {
        let root = tempfile::tempdir().unwrap();
        let config = root.path().join("config");
        std::fs::create_dir_all(config.join(MODULES_DIR)).unwrap();
        std::fs::write(
            config.join(MODULES_DIR).join("shell.lua"),
            r#"return { editor = "nvim" }"#,
        )
        .unwrap();

        let output = render(
            "alt",
            &root.path().join("home"),
            &root.path().join("repo"),
            &config,
            &root.path().join("repo/home/.zshrc.luadot"),
            "export EDITOR=<%= require(\"shell\").editor %>\n",
            &Classes::default(),
            &configuration(),
        )
        .unwrap();

        assert_eq!(
            output.content(),
            &Content::Text("export EDITOR=nvim\n".to_string())
        );
    }

    #[test]
    fn a_bare_name_has_nothing_to_define_it() {
        let root = tempfile::tempdir().unwrap();

        let err = format!(
            "{:#}",
            load(
                root.path(),
                "home/.zshrc.luadot",
                "export EDITOR=<%= editor %>\n",
                &Classes::default(),
            )
            .unwrap_err()
        );

        assert!(err.contains("alt: failed to run"));
        assert!(err.contains("was nil"));
    }

    #[test]
    fn a_broken_template_reports_the_command_and_the_file() {
        let root = tempfile::tempdir().unwrap();

        let err = format!(
            "{:#}",
            load(
                root.path(),
                "home/.zshrc.luadot",
                "fine\n<%= missing() %>\n",
                &Classes::default(),
            )
            .unwrap_err()
        );

        assert!(err.contains("alt: failed to run"));
        assert!(err.contains(".zshrc.luadot"));
        assert!(err.contains(":2:"));
    }
}
