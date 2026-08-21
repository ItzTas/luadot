use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use mlua::Value;

use super::constants::TEMPLATE_FILE;
use crate::files::template_target;
use crate::lua::Shared;
use crate::lua::constants::MODULES_DIR;
use crate::lua::ld::{API, Paths, Surface, install, output, share};
use crate::lua::runtime::{add_module_path, runtime};
use crate::lua::scope::{Output, Scope};
use crate::state::Classes;
use crate::utils;

pub fn load_template(
    command: &str,
    home: &Path,
    repo: &Path,
    dir: &Path,
    classes: &Classes,
    shared: &Shared,
) -> Result<Vec<Output>> {
    let path = dir.join(TEMPLATE_FILE);
    let source = std::fs::read_to_string(&path)
        .with_context(|| format!("{command}: failed to read {}", path.display()))?;
    let dest = destination(command, home, repo, dir)?;
    let config = utils::config_dir()
        .with_context(|| format!("{command}: failed to locate the configuration"))?;

    run(
        command,
        &source,
        &path,
        repo,
        &config,
        Scope::new(dir.to_path_buf(), home.to_path_buf()).with_dest(dest),
        classes,
        shared,
    )
}

#[cfg(test)]
pub fn from_source(dir: &Path, source: &str) -> Result<Vec<Output>> {
    from_classes(dir, source, &Classes::default())
}

#[cfg(test)]
pub fn from_classes(dir: &Path, source: &str, classes: &Classes) -> Result<Vec<Output>> {
    let config = utils::config_dir().context("test: failed to locate the configuration")?;

    from_config(dir, source, &config, classes)
}

#[cfg(test)]
fn from_config(dir: &Path, source: &str, config: &Path, classes: &Classes) -> Result<Vec<Output>> {
    let root = dir.parent().unwrap_or(dir);
    let Some(dest) = template_target(dir) else {
        bail!("test: {} is not a template directory", dir.display());
    };

    run(
        "test",
        source,
        &dir.join(TEMPLATE_FILE),
        root,
        config,
        Scope::new(dir.to_path_buf(), root.to_path_buf()).with_dest(dest),
        classes,
        &std::sync::Arc::new(std::sync::Mutex::new(crate::lua::Config::default())),
    )
}

pub(super) fn destination(command: &str, home: &Path, repo: &Path, dir: &Path) -> Result<PathBuf> {
    let Some(target) = template_target(dir) else {
        bail!("{command}: {} is not a template directory", dir.display());
    };

    utils::system_path(home, repo, &target)
        .with_context(|| format!("{command}: failed to place {}", dir.display()))
}

#[allow(clippy::too_many_arguments)]
fn run(
    command: &str,
    source: &str,
    path: &Path,
    repo: &Path,
    config: &Path,
    scope: Scope,
    classes: &Classes,
    shared: &Shared,
) -> Result<Vec<Output>> {
    let dir = scope.dir().to_path_buf();
    let home = scope.home().to_path_buf();

    let lua = runtime().with_context(|| format!("{command}: failed to start the Lua runtime"))?;
    let paths = Paths::new(&home, config)
        .with_repo(Some(repo))
        .with_dir(&dir);
    install(&lua, Surface::Template, &paths, classes)
        .with_context(|| format!("{command}: failed to install `{API}`"))?;
    share(&lua, shared);
    lua.set_app_data(scope);
    for modules in [dir.as_path(), config].into_iter().rev() {
        add_module_path(&lua, modules)
            .with_context(|| format!("{command}: failed to make {MODULES_DIR}/ requirable"))?;
    }

    let returned: Value = lua
        .load(source)
        .set_name(path.display().to_string())
        .eval()
        .with_context(|| format!("{command}: failed to run {}", path.display()))?;

    if !returned.is_nil() {
        output(&lua, returned)
            .with_context(|| format!("{command}: {} returned an invalid file", path.display()))?;
    }

    let outputs = lua
        .remove_app_data::<Scope>()
        .with_context(|| {
            format!(
                "{command}: the template was lost while running {}",
                path.display()
            )
        })?
        .into_outputs();

    if outputs.is_empty() {
        bail!("{command}: {} produced no file", path.display());
    }

    Ok(outputs)
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use crate::lua::Shared;

    fn configuration() -> Shared {
        Arc::new(Mutex::new(crate::lua::Config::default()))
    }

    use super::*;
    use crate::files::{ConflictPolicy, LinkMode};
    use crate::lua::Content;

    fn template_dir(root: &Path, name: &str) -> PathBuf {
        let dir = root.join(name);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write(dir: &Path, name: &str, contents: &str) {
        std::fs::write(dir.join(name), contents).unwrap();
    }

    fn error(dir: &Path, source: &str) -> String {
        format!("{:#}", from_source(dir, source).unwrap_err())
    }

    #[test]
    fn a_returned_handle_selects_a_file_of_the_template() {
        let root = tempfile::tempdir().unwrap();
        let dir = template_dir(root.path(), ".zshrc.luadot");
        write(&dir, "laptop.zsh", "laptop");

        let outputs = from_source(&dir, r#"return ld.alt.file("laptop.zsh")"#).unwrap();

        assert_eq!(
            outputs,
            vec![Output::new(
                root.path().join(".zshrc"),
                Content::File(dir.join("laptop.zsh")),
                None,
                None,
            )]
        );
    }

    #[test]
    fn a_returned_table_carries_the_whole_descriptor() {
        let root = tempfile::tempdir().unwrap();
        let dir = template_dir(root.path(), ".zshrc.luadot");
        write(&dir, "desktop.zsh", "desktop");

        let outputs = from_source(
            &dir,
            r#"
            return {
              dest = "~/.config/zsh/.zshrc",
              content = ld.alt.file("desktop.zsh"),
              link = "symbolic",
              conflict = "skip",
            }
            "#,
        )
        .unwrap();

        assert_eq!(
            outputs,
            vec![Output::new(
                root.path().join(".config/zsh/.zshrc"),
                Content::File(dir.join("desktop.zsh")),
                Some(LinkMode::Symbolic),
                Some(ConflictPolicy::Skip),
            )]
        );
    }

    #[test]
    fn the_ld_interface_accumulates_every_file_it_declares() {
        let root = tempfile::tempdir().unwrap();
        let dir = template_dir(root.path(), ".zshrc.luadot");
        write(&dir, "laptop.zsh", "laptop");

        let outputs = from_source(
            &dir,
            r#"
            ld.alt.out({ content = ld.alt.file("laptop.zsh") })
            ld.alt.out({ dest = "~/.zprofile", content = "generated\n" })
            "#,
        )
        .unwrap();

        assert_eq!(outputs.len(), 2);
        assert_eq!(outputs[0].dest(), root.path().join(".zshrc"));
        assert_eq!(
            outputs[1].content(),
            &Content::Text("generated\n".to_string())
        );
    }

    #[test]
    fn a_string_is_written_as_generated_content() {
        let root = tempfile::tempdir().unwrap();
        let dir = template_dir(root.path(), ".config/nvim/init.lua.luadot");

        let outputs = from_source(&dir, r#"return "vim.g.mapleader = ' '""#).unwrap();

        assert_eq!(outputs[0].dest(), root.path().join(".config/nvim/init.lua"));
        assert_eq!(
            outputs[0].content(),
            &Content::Text("vim.g.mapleader = ' '".to_string())
        );
    }

    #[test]
    fn render_fills_a_file_of_the_template_with_variables() {
        let root = tempfile::tempdir().unwrap();
        let dir = template_dir(root.path(), ".config/nvim/init.lua.luadot");
        write(
            &dir,
            "init.tmpl.lua",
            r#"return string.format("vim.g.mapleader = %q\n", leader)"#,
        );

        let outputs = from_source(
            &dir,
            r#"return ld.alt.render("init.tmpl.lua", { leader = " " })"#,
        )
        .unwrap();

        assert_eq!(
            outputs[0].content(),
            &Content::Text("vim.g.mapleader = \" \"\n".to_string())
        );
    }

    #[test]
    fn modules_of_the_template_are_requirable() {
        let root = tempfile::tempdir().unwrap();
        let dir = template_dir(root.path(), ".zshrc.luadot");
        std::fs::create_dir_all(dir.join(MODULES_DIR)).unwrap();
        std::fs::write(
            dir.join(MODULES_DIR).join("aliases.lua"),
            r#"return "alias ll='ls -l'\n""#,
        )
        .unwrap();

        let outputs = from_source(&dir, r#"return require("aliases")"#).unwrap();

        assert_eq!(
            outputs[0].content(),
            &Content::Text("alias ll='ls -l'\n".to_string())
        );
    }

    #[test]
    fn a_template_producing_nothing_is_an_error() {
        let root = tempfile::tempdir().unwrap();
        let dir = template_dir(root.path(), ".zshrc.luadot");

        assert!(error(&dir, "local unused = 1").contains("produced no file"));
    }

    #[test]
    fn a_broken_script_reports_the_file() {
        let root = tempfile::tempdir().unwrap();
        let dir = template_dir(root.path(), ".zshrc.luadot");

        let err = error(&dir, "ld.alt.out(");

        assert!(err.contains("failed to run"));
        assert!(err.contains(TEMPLATE_FILE));
    }

    #[test]
    fn load_template_runs_the_file_of_the_directory() {
        let root = tempfile::tempdir().unwrap();
        let home = root.path().join("home");
        let repo = root.path().join("repo");
        let dir = template_dir(&repo, ".zshrc.luadot");
        write(&dir, "laptop.zsh", "laptop");
        write(&dir, TEMPLATE_FILE, r#"return ld.alt.file("laptop.zsh")"#);

        let outputs = load_template(
            "apply",
            &home,
            &repo,
            &dir,
            &Classes::default(),
            &configuration(),
        )
        .unwrap();

        assert_eq!(outputs[0].dest(), home.join(".zshrc"));
        assert_eq!(outputs[0].content(), &Content::File(dir.join("laptop.zsh")));
    }

    #[test]
    fn a_template_selects_a_variant_from_a_class() {
        let root = tempfile::tempdir().unwrap();
        let dir = template_dir(root.path(), ".zshrc.luadot");
        write(&dir, "laptop.zsh", "laptop");
        write(&dir, "desktop.zsh", "desktop");
        let mut classes = Classes::default();
        classes.set("form-factor", "laptop");

        let outputs = from_classes(
            &dir,
            r#"return ld.alt.file(ld.class.get("form-factor") .. ".zsh")"#,
            &classes,
        )
        .unwrap();

        assert_eq!(outputs[0].content(), &Content::File(dir.join("laptop.zsh")));
    }

    #[test]
    fn a_missing_template_file_reports_the_command() {
        let root = tempfile::tempdir().unwrap();
        let repo = root.path().join("repo");
        let dir = template_dir(&repo, ".zshrc.luadot");

        let err = format!(
            "{:#}",
            load_template(
                "apply",
                &root.path().join("home"),
                &repo,
                &dir,
                &Classes::default(),
                &configuration(),
            )
            .unwrap_err()
        );

        assert!(err.contains("apply: failed to read"));
        assert!(err.contains(TEMPLATE_FILE));
    }
}
