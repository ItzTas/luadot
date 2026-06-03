use std::path::Path;

use mlua::{Lua, Table, Value};

use super::handle::FileHandle;

#[derive(Clone, Debug)]
pub struct Host {
    pub name: String,
    pub class: String,
    pub os: String,
    pub arch: String,
    pub cpus: u32,
}

impl Host {
    pub fn detect() -> Self {
        let name = std::fs::read_to_string("/etc/hostname")
            .ok()
            .map(|s| s.trim().to_owned())
            .filter(|s| !s.is_empty())
            .or_else(|| std::env::var("HOSTNAME").ok())
            .or_else(|| std::env::var("HOST").ok())
            .unwrap_or_else(|| "unknown".to_owned());

        let cpus = std::thread::available_parallelism()
            .map(|n| n.get() as u32)
            .unwrap_or(1);

        Self {
            name,
            class: "unknown".to_owned(),
            os: std::env::consts::OS.to_owned(),
            arch: std::env::consts::ARCH.to_owned(),
            cpus,
        }
    }
}

pub fn set_host(lua: &Lua, host: &Host) -> mlua::Result<()> {
    let t = lua.create_table()?;
    t.set("name", host.name.clone())?;
    t.set("class", host.class.clone())?;
    t.set("os", host.os.clone())?;
    t.set("arch", host.arch.clone())?;
    t.set("cpus", host.cpus)?;
    lua.globals().set("host", t)?;
    Ok(())
}

pub fn install_template_api(lua: &Lua, base_dir: impl AsRef<Path>) -> mlua::Result<()> {
    let base = base_dir.as_ref().to_path_buf();

    let file_base = base.clone();
    let file = lua.create_function(move |_, name: String| {
        Ok(FileHandle::new(file_base.join(name)))
    })?;
    lua.globals().set("file", file)?;

    let render_base = base;
    let render = lua.create_function(move |lua, (name, vars): (String, Option<Table>)| {
        let path = render_base.join(&name);
        let src = std::fs::read_to_string(&path).map_err(|e| {
            mlua::Error::external(format!("render: cannot read {}: {e}", path.display()))
        })?;
        render_template(lua, &name, &src, vars)
    })?;
    lua.globals().set("render", render)?;

    Ok(())
}

fn render_template(lua: &Lua, name: &str, src: &str, vars: Option<Table>) -> mlua::Result<String> {
    let env = lua.create_table()?;
    if let Some(vars) = vars {
        for pair in vars.pairs::<Value, Value>() {
            let (k, v) = pair?;
            env.set(k, v)?;
        }
    }

    let mt = lua.create_table()?;
    mt.set("__index", lua.globals())?;
    env.set_metatable(Some(mt))?;

    let out: Value = lua
        .load(src)
        .set_name(name)
        .set_environment(env)
        .eval()?;

    match out {
        Value::String(s) => Ok(s.to_str()?.to_owned()),
        other => Err(mlua::Error::external(format!(
            "render: template {name:?} must return a string, got {}",
            other.type_name()
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_joins_against_base_dir() {
        let lua = super::super::runtime().unwrap();
        install_template_api(&lua, "/src/zshrc.luadot").unwrap();
        let path: String = lua
            .load(r#"return file("zshrc.laptop.zsh").path"#)
            .eval()
            .unwrap();
        assert_eq!(path, "/src/zshrc.luadot/zshrc.laptop.zsh");
    }

    #[test]
    fn render_exposes_vars_and_stdlib() {
        let lua = super::super::runtime().unwrap();
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("t.lua"),
            r#"return string.format("%s/%d", name, n)"#,
        )
        .unwrap();
        install_template_api(&lua, dir.path()).unwrap();
        let out: String = lua
            .load(r#"return render("t.lua", { name = "x", n = 7 })"#)
            .eval()
            .unwrap();
        assert_eq!(out, "x/7");
    }

    #[test]
    fn render_rejects_non_string_result() {
        let lua = super::super::runtime().unwrap();
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("bad.lua"), "return 123").unwrap();
        install_template_api(&lua, dir.path()).unwrap();
        let err = lua
            .load(r#"return render("bad.lua")"#)
            .eval::<String>()
            .unwrap_err();
        assert!(err.to_string().contains("must return a string"));
    }
}
