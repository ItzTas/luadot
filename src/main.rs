mod lua;

use anyhow::Result;

use lua::{FileHandle, Host, install_template_api, runtime, set_host};

fn main() -> Result<()> {
    let rt = runtime()?;
    let host = Host::detect();
    set_host(&rt, &host)?;

    let dir = std::env::temp_dir().join("luadot-demo");
    std::fs::create_dir_all(&dir)?;
    std::fs::write(
        dir.join("greeting.tmpl.lua"),
        r#"return string.format("hello, %s — running on %s", who, host.os)"#,
    )?;
    install_template_api(&rt, &dir)?;

    let descriptor: mlua::Table = rt
        .load(
            r#"
            return {
              picked   = file("greeting.tmpl.lua"),
              rendered = render("greeting.tmpl.lua", { who = host.name }),
            }
            "#,
        )
        .eval()?;

    let picked = descriptor.get::<mlua::AnyUserData>("picked")?;
    let picked = picked.borrow::<FileHandle>()?;
    let rendered: String = descriptor.get("rendered")?;

    println!("luadot — lua integration ok");
    println!("  host     : {} ({}/{}, {} cpus)", host.name, host.os, host.arch, host.cpus);
    println!("  file()   : {}", picked.path().display());
    println!("  render() : {rendered}");

    let _ = std::fs::remove_dir_all(&dir);
    Ok(())
}
