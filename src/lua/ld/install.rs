use mlua::{Lua, Table};

use super::constants::API;
use super::path::Paths;
use super::surface::Surface;
use super::{
    alt, argv, class, cmd, crypt, git, on, opt, path, pkg, print, regex, root, setup, sys,
};
use std::sync::{Arc, Mutex};

use crate::lua::bundled::lpeg;
use crate::lua::{Config, Scope, Shared};
use crate::state::Classes;

type Namespace = fn(&Lua) -> mlua::Result<Table>;

pub fn install(lua: &Lua, surface: Surface, paths: &Paths, classes: &Classes) -> mlua::Result<()> {
    let namespaces: [(&str, Namespace); 10] = [
        (alt::NAMESPACE, alt::table),
        (argv::NAMESPACE, argv::table),
        (cmd::NAMESPACE, cmd::table),
        (crypt::NAMESPACE, crypt::table),
        (on::NAMESPACE, on::table),
        (opt::NAMESPACE, opt::table),
        (pkg::NAMESPACE, pkg::table),
        (print::NAMESPACE, print::table),
        (regex::NAMESPACE, regex::table),
        (sys::NAMESPACE, sys::table),
    ];

    surface.install(lua);
    class::install(lua, classes);
    lua.set_app_data(Shared::new(Mutex::new(Config::default())));
    lua.set_app_data(Scope::new(
        paths.dir().unwrap_or_else(|| paths.config()).to_path_buf(),
        paths.home().to_path_buf(),
    ));

    let ld = root::table(lua)?;
    for (name, namespace) in namespaces {
        ld.set(name, namespace(lua)?)?;
    }
    ld.set(class::NAMESPACE, class::table(lua, classes)?)?;
    ld.set(git::NAMESPACE, git::table(lua, paths)?)?;
    ld.set(path::NAMESPACE, path::table(lua, paths)?)?;
    ld.set(setup::NAMESPACE, setup::table(lua, paths)?)?;
    lpeg::install(lua, &ld)?;

    lua.globals().set(API, ld)
}

pub fn share(lua: &Lua, config: &Shared) {
    lua.set_app_data(Arc::clone(config));
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;
    use crate::lua::from_source;

    use crate::lua::runtime::runtime;
    use crate::state::Classes;

    const EVERY_CALL: &str = r#"
        assert(type(ld.rules) == "function", "rules is missing")
        for _, name in ipairs({ "out", "file", "render", "expand", "read", "exists", "glob", "json" }) do
          assert(type(ld.alt[name]) == "function", "alt." .. name .. " is missing")
        end
        assert(type(getmetatable(ld.git).__call) == "function", "git is not callable")
        for _, name in ipairs({ "backup", "backup_age", "backup_dir", "backup_keep", "conflict", "link", "passphrase_warn", "pkg_warn", "repo_dir" }) do
          assert(type(ld.opt[name]) == "function", "opt." .. name .. " is missing")
        end
        assert(type(getmetatable(ld.opt).__call) == "function", "opt is not callable")
        for _, name in ipairs({ "backend", "lock" }) do
          assert(type(ld.crypt[name]) == "function", "crypt." .. name .. " is missing")
        end
        assert(type(getmetatable(ld.crypt).__call) == "function", "crypt is not callable")
        assert(type(ld.pkg.install) == "function", "pkg.install is missing")
        for _, name in ipairs({ "list", "all" }) do
          assert(type(ld.setup[name]) == "function", "setup." .. name .. " is missing")
        end
        assert(type(getmetatable(ld.setup).__call) == "function", "setup is not callable")
        assert(type(ld.class.get) == "function", "class.get is missing")
        assert(type(getmetatable(ld.class).__call) == "function", "class is not callable")
        assert(type(getmetatable(ld.cmd).__call) == "function", "cmd is not callable")
        assert(type(ld.cmd.ls) == "function", "cmd is not indexable")
        assert(type(getmetatable(ld.print).__call) == "function", "print is not callable")
        for _, name in ipairs({ "note", "warn", "error", "section", "entry", "field" }) do
          assert(type(ld.print[name]) == "function", "print." .. name .. " is missing")
        end
        for _, name in ipairs({ "diff", "status" }) do
          assert(type(ld.on[name]) == "function", "on." .. name .. " is missing")
        end
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
        for _, name in ipairs({ "test", "match", "find", "gmatch", "gsub", "split", "escape" }) do
          assert(type(ld.regex[name]) == "function", "regex." .. name .. " is missing")
        end
        assert(type(ld.lpeg.P) == "function", "lpeg is missing")
        assert(type(ld.re.match) == "function", "re is missing")
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
    fn the_bootstrap_carries_every_call() {
        exec(Surface::Bootstrap, EVERY_CALL);
    }

    #[test]
    fn a_setup_script_carries_every_call() {
        exec(Surface::Setup, EVERY_CALL);
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
            ld.rules({ { match = "*.swp", ignore = true } })
            ld.opt.link("symbolic")
            ld.opt({ link = "symbolic" })
            ld.opt.conflict("skip")
            ld.class({ name = "form-factor" })
            ld.crypt.backend("gpg")
            ld.crypt.lock({ recipients = "age1example", identity = "~/.keys/age.txt" })
            ld.crypt.lock("passphrase")
            ld.crypt({ backend = "age" })
            ld.on.diff({ summary = false })
            ld.on.status({ summary = false })
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
    fn the_alternatives_resolve_against_the_directory_of_the_surface() {
        exec(
            Surface::Config,
            r#"
            assert(ld.alt.exists("laptop.zsh") == false, "alt.exists answered for a file that is not there")
            assert(#ld.alt.glob("*.zsh") == 0, "alt.glob found something")
            assert(ld.alt.json({ n = 1 }) == '{\n  "n": 1\n}', "alt.json needs no directory")

            for _, name in ipairs({ "file", "read", "render", "expand" }) do
              local ok, err = pcall(ld.alt[name], "laptop.zsh")
              assert(not ok, "alt." .. name .. " answered without a file")
              assert(
                tostring(err):find("/home/u/.config/luadot", 1, true),
                "alt." .. name .. " looked outside the surface: " .. tostring(err)
              )
            end
            "#,
        );
    }

    #[test]
    fn a_configuration_call_from_a_template_still_does_nothing() {
        exec(
            Surface::Template,
            r#"
            ld.opt.pkg_warn(false)
            ld.crypt.lock({ recipients = "age1example" })
            "#,
        );
    }

    #[test]
    fn the_configuration_surface_reports_a_setup_without_a_repository() {
        let err = format!("{:#}", from_source(r#"ld.setup("ufw")"#).unwrap_err());

        assert!(err.contains("`ld.setup`: no repository set"));
    }
}
