use mlua::{Lua, Table};

use super::super::table::options;
use super::constants::{NAMESPACE, SETTERS};

pub fn table(lua: &Lua) -> mlua::Result<Table> {
    options(lua, NAMESPACE, &SETTERS, "crypt option")
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::crypt::Backend;
    use crate::lua::from_source;

    #[test]
    fn the_call_form_sets_several_options_at_once() {
        let config = from_source(
            r#"
            ld.crypt({
              backend = "gpg",
              recipients = { "me@example.com" },
              identity = "~/.keys/private.asc",
            })
            "#,
        )
        .unwrap();

        assert_eq!(config.crypt_backend(), Backend::Gpg);
        assert_eq!(config.crypt_recipients(), ["me@example.com"]);
        assert_eq!(
            config.crypt_identity(),
            Some(Path::new("~/.keys/private.asc"))
        );
    }

    #[test]
    fn the_call_form_only_touches_the_options_it_carries() {
        let config = from_source(
            r#"
            ld.crypt.backend("gpg")
            ld.crypt({ recipients = "me@example.com" })
            "#,
        )
        .unwrap();

        assert_eq!(config.crypt_backend(), Backend::Gpg);
        assert_eq!(config.crypt_recipients(), ["me@example.com"]);
    }

    #[test]
    fn an_unknown_option_is_refused() {
        let err = format!(
            "{:#}",
            from_source(r#"ld.crypt({ passphrase = "hunter2" })"#).unwrap_err()
        );

        assert!(err.contains("unknown crypt option `passphrase`"));
        assert!(err.contains("backend, identity, recipients"));
    }
}
