use mlua::{Function, Lua};

use super::constants::READ;
use super::file::{read, resolve};

pub fn function(lua: &Lua) -> mlua::Result<Function> {
    lua.create_function(|lua, name: String| {
        let path = resolve(lua, &name, READ)?;
        read(READ, &path)
    })
}

#[cfg(test)]
mod tests {
    use super::super::fixture::template;
    use crate::lua::{Content, from_template};

    #[test]
    fn yields_the_bytes_of_a_file() {
        let root = tempfile::tempdir().unwrap();
        let dir = template(root.path());
        std::fs::write(dir.join("aliases.zsh"), "alias ll='ls -l'\n").unwrap();

        let outputs = from_template(&dir, r#"return ld.alt.read("aliases.zsh")"#).unwrap();

        assert_eq!(
            outputs[0].content(),
            &Content::Text("alias ll='ls -l'\n".to_string())
        );
    }
}
