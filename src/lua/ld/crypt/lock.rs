use mlua::{Lua, Table, Value};

use super::super::parse::external;
use super::super::surface::{self, Surface};
use super::super::value::{expected, flag, keys, path, text};
use super::constants::{
    IDENTITY, IDENTITY_COMMAND, LOCK, LOCK_CONFLICT, LOCK_KEYS, LOCK_KIND, NAMESPACE, PASSPHRASE,
    RECIPIENTS,
};
use crate::crypt::{Provider, Secrets};
use crate::lua::Config;

pub fn set(lua: &Lua, value: Value) -> mlua::Result<()> {
    if surface::inert(lua, &format!("{NAMESPACE}.{LOCK}"), Surface::Config) {
        return Ok(());
    }

    let secrets = secrets(&value)?;
    Config::building(lua)?.set_crypt_secrets(secrets);
    Ok(())
}

fn secrets(value: &Value) -> mlua::Result<Secrets> {
    match value {
        Value::String(_) => passphrase(value),
        Value::Table(options) => keyed(options),
        _ => Err(expected(NAMESPACE, LOCK, LOCK_KIND)),
    }
}

fn passphrase(value: &Value) -> mlua::Result<Secrets> {
    let word = text(NAMESPACE, value, LOCK)?;
    if word != PASSPHRASE {
        return Err(expected(NAMESPACE, LOCK, LOCK_KIND));
    }

    Ok(Secrets::Passphrase)
}

fn keyed(options: &Table) -> mlua::Result<Secrets> {
    let recipients = match options.get::<Value>(RECIPIENTS)? {
        Value::Nil => Vec::new(),
        value => keys(LOCK_KEYS, &value, RECIPIENTS)?,
    };

    let identity = match options.get::<Value>(IDENTITY)? {
        Value::Nil => None,
        value => Some(path(LOCK_KEYS, &value, IDENTITY, "a path")?),
    };

    let identity_command = match options.get::<Value>(IDENTITY_COMMAND)? {
        Value::Nil => None,
        value => Some(provider(&value)?),
    };

    let passphrase = match options.get::<Value>(PASSPHRASE)? {
        Value::Nil => false,
        value => flag(LOCK_KEYS, &value, PASSPHRASE)?,
    };

    if !passphrase {
        return Ok(Secrets::Keys {
            recipients,
            identity,
            identity_command,
        });
    }
    if !recipients.is_empty() || identity.is_some() || identity_command.is_some() {
        return Err(external(LOCK_CONFLICT.to_string()));
    }

    Ok(Secrets::Passphrase)
}

fn provider(value: &Value) -> mlua::Result<Provider> {
    let kind = "a command line or a list of a program and its arguments";
    match value {
        Value::String(_) => line(value, kind),
        Value::Table(_) => keys(LOCK_KEYS, value, IDENTITY_COMMAND)
            .map(Provider::Program)
            .map_err(|_| expected(LOCK_KEYS, IDENTITY_COMMAND, kind)),
        _ => Err(expected(LOCK_KEYS, IDENTITY_COMMAND, kind)),
    }
}

fn line(value: &Value, kind: &str) -> mlua::Result<Provider> {
    let line = text(LOCK_KEYS, value, IDENTITY_COMMAND)?;
    if line.trim().is_empty() {
        return Err(expected(LOCK_KEYS, IDENTITY_COMMAND, kind));
    }

    Ok(Provider::Line(line))
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::crypt::{Provider, Secrets};
    use crate::lua::from_source;

    #[test]
    fn defaults_to_keys_without_any() {
        let config = from_source("local unused = 1").unwrap();

        assert_eq!(config.crypt_secrets(), &Secrets::default());
    }

    #[test]
    fn the_word_passphrase_picks_the_passphrase_lock() {
        let config = from_source(r#"ld.crypt.lock("passphrase")"#).unwrap();

        assert_eq!(config.crypt_secrets(), &Secrets::Passphrase);
    }

    #[test]
    fn a_table_carries_the_keys_the_lock_needs() {
        let config = from_source(
            r#"
            ld.crypt.lock({
              recipients = { "age1first", "age1second" },
              identity = "~/.keys/age.txt",
              identity_command = "pass show age/key",
            })
            "#,
        )
        .unwrap();

        assert_eq!(
            config.crypt_secrets(),
            &Secrets::Keys {
                recipients: vec!["age1first".to_string(), "age1second".to_string()],
                identity: Some(Path::new("~/.keys/age.txt").to_path_buf()),
                identity_command: Some(Provider::Line("pass show age/key".to_string())),
            }
        );
    }

    #[test]
    fn the_table_form_reaches_the_passphrase_too() {
        let config = from_source("ld.crypt.lock({ passphrase = true })").unwrap();

        assert_eq!(config.crypt_secrets(), &Secrets::Passphrase);
    }

    #[test]
    fn a_passphrase_turned_off_leaves_the_keys_alone() {
        let config =
            from_source(r#"ld.crypt.lock({ passphrase = false, recipients = "age1example" })"#)
                .unwrap();

        assert_eq!(config.crypt_secrets().recipients(), ["age1example"]);
    }

    #[test]
    fn a_passphrase_beside_keys_is_refused() {
        let err = format!(
            "{:#}",
            from_source(r#"ld.crypt.lock({ passphrase = true, recipients = "age1example" })"#)
                .unwrap_err()
        );

        assert!(err.contains("locks with a passphrase or with keys, never both"));
    }

    #[test]
    fn a_single_recipient_needs_no_list() {
        let config = from_source(r#"ld.crypt.lock({ recipients = "age1example" })"#).unwrap();

        assert_eq!(config.crypt_secrets().recipients(), ["age1example"]);
    }

    #[test]
    fn an_identity_command_takes_a_program_and_its_arguments() {
        let config = from_source(
            r#"ld.crypt.lock({ identity_command = { "op", "read", "op://vault/age/key" } })"#,
        )
        .unwrap();

        let Secrets::Keys {
            identity_command, ..
        } = config.crypt_secrets()
        else {
            panic!("expected the key form");
        };

        assert_eq!(
            identity_command.as_ref(),
            Some(&Provider::Program(vec![
                "op".to_string(),
                "read".to_string(),
                "op://vault/age/key".to_string(),
            ]))
        );
    }

    #[test]
    fn the_last_call_wins() {
        let config = from_source(
            r#"
            ld.crypt.lock({ recipients = "age1example" })
            ld.crypt.lock("passphrase")
            "#,
        )
        .unwrap();

        assert_eq!(config.crypt_secrets(), &Secrets::Passphrase);
    }

    #[test]
    fn rejects_a_word_that_names_no_lock() {
        let err = format!("{:#}", from_source(r#"ld.crypt.lock("keys")"#).unwrap_err());

        assert!(err.contains("`ld.crypt.lock` takes"));
    }
}
