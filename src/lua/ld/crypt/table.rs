use mlua::{Lua, Table};

use super::super::table::options;
use super::constants::{NAMESPACE, SETTERS};

pub fn table(lua: &Lua) -> mlua::Result<Table> {
    options(lua, NAMESPACE, &SETTERS, "crypt option")
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::crypt::{Backend, Key, Secrets};
    use crate::lua::from_source;

    #[test]
    fn the_call_form_sets_every_option() {
        let config = from_source(
            r#"
            ld.crypt({
              backend = "gpg",
              lock = {
                recipients = { "me@example.com" },
                identity = "~/.keys/private.asc",
              },
            })
            "#,
        )
        .unwrap();

        assert_eq!(config.crypt_backend(), Backend::Gpg);
        assert_eq!(
            config.crypt_secrets(),
            &Secrets::Keys {
                recipients: vec!["me@example.com".to_string()],
                identity: Some(Key::File(Path::new("~/.keys/private.asc").to_path_buf())),
            }
        );
    }
}
