use mlua::{Lua, Table};

use super::constants::API;
use super::path::Paths;
use super::surface::Surface;
use super::{
    alt, argv, class, cmd, crypt, doc, fs, git, json, on, opt, path, pkg, print, regex, root, rtp,
    setup,
};
use std::sync::{Arc, Mutex};

use crate::lua::bundled::lpeg;
use crate::lua::{Config, Scope, Shared};
use crate::state::Classes;

type Namespace = fn(&Lua) -> mlua::Result<Table>;

pub fn install(lua: &Lua, surface: Surface, paths: &Paths, classes: &Classes) -> mlua::Result<()> {
    let namespaces: [(&str, Namespace); 13] = [
        (alt::NAMESPACE, alt::table),
        (argv::NAMESPACE, argv::table),
        (cmd::NAMESPACE, cmd::table),
        (crypt::NAMESPACE, crypt::table),
        (doc::NAMESPACE, doc::table),
        (fs::NAMESPACE, fs::table),
        (json::NAMESPACE, json::table),
        (on::NAMESPACE, on::table),
        (opt::NAMESPACE, opt::table),
        (pkg::NAMESPACE, pkg::table),
        (print::NAMESPACE, print::table),
        (regex::NAMESPACE, regex::table),
        (rtp::NAMESPACE, rtp::table),
    ];

    surface.install(lua);
    class::install(lua, classes);
    lua.set_app_data(Shared::new(Mutex::new(Config::default())));
    lua.set_app_data(Scope::new(
        paths.dir().unwrap_or_else(|| paths.config()).to_path_buf(),
        paths.home().to_path_buf(),
    ));

    let ld = root::table(lua, surface)?;
    for (name, namespace) in namespaces {
        ld.set(name, namespace(lua)?)?;
    }
    ld.set(class::NAMESPACE, class::table(lua)?)?;
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
    use crate::lua::runtime::runtime;
    use crate::state::Classes;

    const EVERY_CALL: &str = r#"
        assert(type(ld.rules) == "function", "rules is missing")
        assert(type(ld.task) == "function", "task is missing")
        assert(ld.surface == "bootstrap", "surface is wrong: " .. tostring(ld.surface))
        for _, name in ipairs({ "out", "file", "render", "expand", "read", "exists", "glob", "concat", "json" }) do
          assert(type(ld.alt[name]) == "function", "alt." .. name .. " is missing")
        end
        assert(type(getmetatable(ld.git).__call) == "function", "git is not callable")
        for _, name in ipairs({ "clone", "at" }) do
          assert(type(ld.git[name]) == "function", "git." .. name .. " is missing")
        end
        for _, name in ipairs({ "backup", "backup_age", "backup_dir", "backup_keep", "conflict", "link", "passphrase_warn", "pkg_warn", "repo_dir" }) do
          assert(type(ld.opt[name]) == "function", "opt." .. name .. " is missing")
        end
        assert(type(getmetatable(ld.opt).__call) == "function", "opt is not callable")
        for _, name in ipairs({ "backend", "lock" }) do
          assert(type(ld.crypt[name]) == "function", "crypt." .. name .. " is missing")
        end
        assert(type(getmetatable(ld.crypt).__call) == "function", "crypt is not callable")
        assert(type(ld.pkg.install) == "function", "pkg.install is missing")
        assert(type(ld.rtp.add) == "function", "rtp.add is missing")
        assert(type(ld.doc.page) == "function", "doc.page is missing")
        for _, name in ipairs({ "encode", "decode" }) do
          assert(type(ld.json[name]) == "function", "json." .. name .. " is missing")
        end
        assert(type(ld.json.null) == "userdata", "json.null is missing")
        for _, name in ipairs({ "exists", "is_dir", "mkdir", "ls", "rm", "read", "write" }) do
          assert(type(ld.fs[name]) == "function", "fs." .. name .. " is missing")
        end
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
        for _, name in ipairs({ "add", "apply", "bootstrap", "cd", "class", "clone", "config", "diff", "edit", "exec", "git", "init", "push", "rekey", "restore", "rm", "setup", "status", "sync" }) do
          assert(type(ld.on[name]) == "function", "on." .. name .. " is missing")
        end
        for _, name in ipairs({ "alt", "new" }) do
          assert(type(ld.on.tmpl[name]) == "function", "on.tmpl." .. name .. " is missing")
        end
        assert(type(ld.argv.name) == "string", "argv.name is missing")
        assert(type(ld.argv.args) == "table", "argv.args is missing")
        assert(type(ld.path.home) == "string", "path.home is missing")
        assert(type(ld.path.config) == "string", "path.config is missing")
        assert(type(ld.path.data) == "string", "path.data is missing")
        for _, name in ipairs({ "test", "match", "find", "gmatch", "gsub", "split", "escape" }) do
          assert(type(ld.regex[name]) == "function", "regex." .. name .. " is missing")
        end
        assert(type(ld.lpeg.P) == "function", "lpeg is missing")
        assert(type(ld.re.match) == "function", "re is missing")
    "#;

    fn paths() -> Paths {
        Paths::new(
            Path::new("/home/u"),
            Path::new("/home/u/.config/luadot"),
            Path::new("/home/u/.local/share/luadot"),
        )
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

    fn undocumented(table: &Table, prefix: &str, found: &mut Vec<String>) {
        for pair in table.clone().pairs::<String, mlua::Value>() {
            let (name, value) = pair.unwrap();
            let path = match prefix.is_empty() {
                true => name,
                false => format!("{prefix}.{name}"),
            };

            if let Some(table) = value.as_table() {
                undocumented(table, &path, found);
                continue;
            }

            if value.is_function() && !crate::cli::documented(&path) {
                found.push(path);
            }
        }
    }

    #[test]
    fn every_call_is_documented() {
        let lua = runtime().unwrap();
        install(&lua, Surface::Bootstrap, &paths(), &Classes::default()).unwrap();

        let mut found = Vec::new();
        undocumented(&lua.globals().get::<Table>(API).unwrap(), "", &mut found);

        assert!(found.is_empty(), "undocumented: {found:?}");
    }

    #[test]
    fn a_call_outside_the_config_lands() {
        let lua = runtime().unwrap();
        install(&lua, Surface::Bootstrap, &paths(), &Classes::default()).unwrap();

        lua.load(
            r#"
            ld.opt.link("symbolic")
            ld.rules({ { match = "*.swp", track = "never" } })
            "#,
        )
        .exec()
        .unwrap();

        let shared = Config::shared(&lua).unwrap();
        let config = shared.lock().unwrap();

        assert_eq!(
            config.link_mode(Path::new(".bashrc")),
            crate::files::LinkMode::Symbolic
        );
        assert!(config.is_ignored(Path::new(".vimrc.swp")));
    }
}
