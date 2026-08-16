use mlua::{Lua, Table};

use super::constants::API;
use super::path::Paths;
use super::surface::Surface;
use super::{alt, argv, class, cmd, git, opt, path, pkg, root, setup, sys};
use crate::lua::Config;
use crate::state::Classes;

type Namespace = fn(&Lua) -> mlua::Result<Table>;

pub fn install(lua: &Lua, surface: Surface, paths: &Paths, classes: &Classes) -> mlua::Result<()> {
    let namespaces: [(&str, Namespace); 6] = [
        (alt::NAMESPACE, alt::table),
        (argv::NAMESPACE, argv::table),
        (cmd::NAMESPACE, cmd::table),
        (opt::NAMESPACE, opt::table),
        (pkg::NAMESPACE, pkg::table),
        (sys::NAMESPACE, sys::table),
    ];

    surface.install(lua);
    class::install(lua, classes);
    lua.set_app_data(Config::default());

    let ld = root::table(lua)?;
    for (name, namespace) in namespaces {
        ld.set(name, namespace(lua)?)?;
    }
    ld.set(class::NAMESPACE, class::table(lua, classes)?)?;
    ld.set(git::NAMESPACE, git::table(lua, paths)?)?;
    ld.set(path::NAMESPACE, path::table(lua, paths)?)?;
    ld.set(setup::NAMESPACE, setup::table(lua, paths)?)?;

    lua.globals().set(API, ld)
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;
    use crate::lua::from_source;
    use crate::lua::from_template;
    use crate::lua::runtime::runtime;
    use crate::state::Classes;

    const EVERY_CALL: &str = r#"
        assert(type(ld.rules) == "function", "rules is missing")
        for _, name in ipairs({ "out", "file", "render" }) do
          assert(type(ld.alt[name]) == "function", "alt." .. name .. " is missing")
        end
        for _, name in ipairs({ "ignore", "conflict" }) do
          assert(type(ld.git[name]) == "function", "git." .. name .. " is missing")
        end
        assert(type(getmetatable(ld.git).__call) == "function", "git is not callable")
        for _, name in ipairs({ "backup", "backup_dir", "backup_keep", "link", "pkg_warn", "repo_dir" }) do
          assert(type(ld.opt[name]) == "function", "opt." .. name .. " is missing")
        end
        assert(type(getmetatable(ld.opt).__call) == "function", "opt is not callable")
        assert(type(ld.pkg.install) == "function", "pkg.install is missing")
        for _, name in ipairs({ "list", "all" }) do
          assert(type(ld.setup[name]) == "function", "setup." .. name .. " is missing")
        end
        assert(type(getmetatable(ld.setup).__call) == "function", "setup is not callable")
        assert(type(ld.class.get) == "function", "class.get is missing")
        assert(type(getmetatable(ld.class).__call) == "function", "class is not callable")
        assert(type(getmetatable(ld.cmd).__call) == "function", "cmd is not callable")
        assert(type(ld.cmd.ls) == "function", "cmd is not indexable")
        assert(type(ld.argv.name) == "string", "argv.name is missing")
        assert(type(ld.argv.args) == "table", "argv.args is missing")
        assert(ld.host == nil, "host leaked into the root")
        for _, name in ipairs({ "name", "os", "arch" }) do
          assert(type(ld.sys.host[name]) == "string", "sys.host." .. name .. " is missing")
        end
        for _, name in ipairs({ "vendor", "name", "driver" }) do
          assert(type(ld.sys.gpu[name]) == "string", "sys.gpu." .. name .. " is missing")
        end
        assert(type(ld.sys.ram) == "number", "sys.ram is missing")
        assert(type(ld.sys.has_battery()) == "boolean", "sys.has_battery is missing")
        assert(type(ld.path.home) == "string", "path.home is missing")
        assert(type(ld.path.config) == "string", "path.config is missing")
    "#;

    fn paths() -> Paths {
        Paths::new(Path::new("/home/u"), Path::new("/home/u/.config/luadot"))
            .with_repo(Some(Path::new("/data/repo")))
    }

    fn exec(surface: Surface, source: &str) {
        let lua = runtime().unwrap();
        install(&lua, surface, &paths(), &Classes::default()).unwrap();

        lua.load(source).exec().unwrap();
    }

    #[test]
    fn the_configuration_carries_every_call() {
        assert!(from_source(EVERY_CALL).is_ok());
    }

    #[test]
    fn the_bootstrap_carries_every_call() {
        exec(Surface::Bootstrap, EVERY_CALL);
    }

    #[test]
    fn a_setup_script_carries_every_call() {
        exec(Surface::Setup, EVERY_CALL);
    }

    #[test]
    fn a_template_carries_every_call() {
        let root = tempfile::tempdir().unwrap();
        let dir = root.path().join(".zshrc.luadot");
        std::fs::create_dir_all(&dir).unwrap();

        assert!(from_template(&dir, &format!("{EVERY_CALL}\nreturn \"ok\"")).is_ok());
    }

    #[test]
    fn the_paths_of_the_run_are_the_ones_installed() {
        exec(
            Surface::Bootstrap,
            r#"
            assert(ld.path.home == "/home/u", "path.home is wrong")
            assert(ld.path.config == "/home/u/.config/luadot", "path.config is wrong")
            assert(ld.path.repo == "/data/repo", "path.repo is wrong")
            assert(ld.path.dir == nil, "path.dir belongs to a template")
            "#,
        );
    }

    #[test]
    fn a_configuration_call_away_from_the_configuration_does_nothing() {
        exec(
            Surface::Bootstrap,
            r#"
            ld.opt.pkg_warn(false)
            ld.rules({ { match = ".ssh/**", link = "symbolic" } })
            ld.opt.link("symbolic")
            ld.opt({ link = "symbolic" })
            ld.git.ignore("*.swp")
            ld.git.conflict("skip")
            ld.class({ name = "form-factor" })
            "#,
        );
    }

    #[test]
    fn the_classes_of_the_machine_are_readable_from_every_surface() {
        let source = r#"assert(ld.class.get("form-factor") == "laptop", "the class is missing")"#;
        let mut classes = Classes::default();
        classes.set("form-factor", "laptop");

        for surface in [Surface::Bootstrap, Surface::Setup, Surface::Template] {
            let lua = runtime().unwrap();
            install(&lua, surface, &paths(), &classes).unwrap();

            lua.load(source).exec().unwrap();
        }

        assert!(crate::lua::from_classes(source, &classes).is_ok());
    }

    #[test]
    fn a_template_call_away_from_a_template_yields_nothing() {
        exec(
            Surface::Config,
            r#"
            ld.opt.pkg_warn(false)
            assert(ld.alt.file("laptop.zsh") == nil, "alt.file produced a handle")
            assert(ld.alt.render("init.tmpl.lua") == nil, "alt.render produced a string")
            ld.alt.out({ content = "generated\n" })
            "#,
        );
    }

    #[test]
    fn the_configuration_surface_reports_a_setup_without_a_repository() {
        let err = format!("{:#}", from_source(r#"ld.setup("ufw")"#).unwrap_err());

        assert!(err.contains("`ld.setup`: no repository set"));
    }
}
