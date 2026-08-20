use std::path::PathBuf;

use mlua::{Lua, Table, Value};

use super::super::parse::external;
use super::super::surface::{self, Surface};
use super::super::value::{choice, expected, keys, text};
use super::constants::{
    FILE_ALONE, IDENTITY, IDENTITY_KEYS, IDENTITY_KIND, IDENTITY_TYPE, IDENTITY_TYPES, Kind, LOCK,
    LOCK_KEYS, LOCK_KIND, NAMESPACE, PASSPHRASE, RECIPIENTS, TYPE,
};
use crate::crypt::{Key, Provider, Secrets};
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
        value => Some(key(&value)?),
    };

    Ok(Secrets::Keys {
        recipients,
        identity,
    })
}

fn key(value: &Value) -> mlua::Result<Key> {
    match value {
        Value::String(_) => Ok(guessed(text(LOCK_KEYS, value, IDENTITY)?)),
        Value::Table(options) => keyed_identity(options),
        _ => Err(expected(LOCK_KEYS, IDENTITY, IDENTITY_KIND)),
    }
}

fn guessed(written: String) -> Key {
    match written.trim().contains(char::is_whitespace) {
        true => Key::Command(Provider::Line(written)),
        false => Key::File(PathBuf::from(written)),
    }
}

fn keyed_identity(options: &Table) -> mlua::Result<Key> {
    let words = keys(IDENTITY_KEYS, &Value::Table(options.clone()), IDENTITY)
        .map_err(|_| expected(LOCK_KEYS, IDENTITY, IDENTITY_KIND))?;

    match options.get::<Value>(TYPE)? {
        Value::Nil => Ok(program(words)),
        value => typed(
            choice(IDENTITY_KEYS, &value, TYPE, &IDENTITY_TYPES, IDENTITY_TYPE)?,
            words,
        ),
    }
}

fn typed(kind: Kind, mut words: Vec<String>) -> mlua::Result<Key> {
    if kind == Kind::Command {
        return Ok(program(words));
    }
    if words.len() != 1 {
        return Err(external(FILE_ALONE.to_string()));
    }

    Ok(Key::File(PathBuf::from(words.remove(0))))
}

fn program(mut words: Vec<String>) -> Key {
    match words.len() {
        1 => Key::Command(Provider::Line(words.remove(0))),
        _ => Key::Command(Provider::Program(words)),
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::crypt::{Key, Provider, Secrets};
    use crate::lua::from_source;

    fn identity(source: &str) -> Key {
        let config = from_source(source).unwrap();
        let Secrets::Keys { identity, .. } = config.crypt_secrets() else {
            panic!("expected the key form");
        };

        identity.clone().expect("expected an identity")
    }

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
    fn a_table_carries_the_recipients_and_the_identity() {
        let config = from_source(
            r#"
            ld.crypt.lock({
              recipients = { "age1first", "age1second" },
              identity = "~/.keys/age.txt",
            })
            "#,
        )
        .unwrap();

        assert_eq!(
            config.crypt_secrets(),
            &Secrets::Keys {
                recipients: vec!["age1first".to_string(), "age1second".to_string()],
                identity: Some(Key::File(PathBuf::from("~/.keys/age.txt"))),
            }
        );
    }

    #[test]
    fn a_written_identity_without_a_space_is_a_path() {
        assert_eq!(
            identity(r#"ld.crypt.lock({ identity = "~/.keys/age.txt" })"#),
            Key::File(PathBuf::from("~/.keys/age.txt"))
        );
    }

    #[test]
    fn a_written_identity_carrying_a_space_is_a_command() {
        assert_eq!(
            identity(r#"ld.crypt.lock({ identity = "pass show age/key" })"#),
            Key::Command(Provider::Line("pass show age/key".to_string()))
        );
    }

    #[test]
    fn a_type_names_what_the_guess_would_have_missed() {
        assert_eq!(
            identity(r#"ld.crypt.lock({ identity = { type = "file", "/mnt/my key.txt" } })"#),
            Key::File(PathBuf::from("/mnt/my key.txt"))
        );
        assert_eq!(
            identity(r#"ld.crypt.lock({ identity = { type = "command", "unlock-key" } })"#),
            Key::Command(Provider::Line("unlock-key".to_string()))
        );
    }

    #[test]
    fn several_words_are_a_program_and_its_arguments() {
        let expected = Key::Command(Provider::Program(vec![
            "op".to_string(),
            "read".to_string(),
            "op://vault/age/key".to_string(),
        ]));

        assert_eq!(
            identity(r#"ld.crypt.lock({ identity = { "op", "read", "op://vault/age/key" } })"#),
            expected
        );
        assert_eq!(
            identity(
                r#"ld.crypt.lock({ identity = { type = "command", "op", "read", "op://vault/age/key" } })"#
            ),
            expected
        );
    }

    #[test]
    fn rejects_a_type_that_names_nothing() {
        let err = format!(
            "{:#}",
            from_source(r#"ld.crypt.lock({ identity = { type = "keyring", "x" } })"#).unwrap_err()
        );

        assert!(err.contains("unknown identity type `keyring`"));
        assert!(err.contains("command, file"));
    }

    #[test]
    fn rejects_a_word_that_names_no_lock() {
        let err = format!("{:#}", from_source(r#"ld.crypt.lock("keys")"#).unwrap_err());

        assert!(err.contains("`ld.crypt.lock` takes"));
    }
}
