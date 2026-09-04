use mlua::{Function, Lua, Value};

use super::super::surface::{self, Surface};
use super::super::value::path;
use super::constants::{NAMESPACE, PAGE};
use crate::lua::{Config, Scope};
use crate::utils::expand;

pub fn function(lua: &Lua) -> mlua::Result<Function> {
    lua.create_function(|lua, value: Value| {
        if surface::inert(lua, &format!("{NAMESPACE}.{PAGE}"), Surface::Config) {
            return Ok(());
        }

        let raw = path(NAMESPACE, &value, PAGE, "a page")?;
        let home = Scope::building(lua)?.home().to_path_buf();

        Config::building(lua, |config| config.add_doc_page(expand(&home, &raw)))
    })
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::lua::from_source;

    #[test]
    fn a_page_is_kept_once() {
        let config = from_source(
            r#"
            ld.doc.page("~/plugins/lazyld/docs/lazyld.md")
            ld.doc.page("plugins/lazyld/docs/lazyld.md")
            ld.doc.page("/usr/share/doc/other.md")
            "#,
        )
        .unwrap();

        let home = crate::utils::home_dir().unwrap();
        assert_eq!(
            config.doc_pages(),
            [
                home.join("plugins/lazyld/docs/lazyld.md"),
                PathBuf::from("/usr/share/doc/other.md"),
            ]
        );
    }
}
