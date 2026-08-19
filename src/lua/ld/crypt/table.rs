use mlua::{Lua, Table};

use super::super::table::options;
use super::constants::{NAMESPACE, SETTERS};

pub fn table(lua: &Lua) -> mlua::Result<Table> {
    options(lua, NAMESPACE, &SETTERS, "crypt option")
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::crypt::{Backend, Secrets};
    use crate::lua::from_source;

    #[test]
    fn the_call_form_sets_several_options_at_once() {
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
                identity: Some(Path::new("~/.keys/private.asc").to_path_buf()),
                identity_command: None,
            }
        );
    }

    #[test]
    fn the_call_form_only_touches_the_options_it_carries() {
        let config = from_source(
            r#"
            ld.crypt.backend("gpg")
            ld.crypt({ lock = "passphrase" })
            "#,
        )
        .unwrap();

        assert_eq!(config.crypt_backend(), Backend::Gpg);
        assert_eq!(config.crypt_secrets(), &Secrets::Passphrase);
    }

    #[test]
    fn an_unknown_option_is_refused() {
        let err = format!(
            "{:#}",
            from_source(r#"ld.crypt({ secret = "hunter2" })"#).unwrap_err()
        );

        assert!(err.contains("unknown crypt option `secret`"));
        assert!(err.contains("backend, lock"));
    }
}
