use mlua::{Lua, Value};

use super::super::surface::{self, Surface};
use super::super::value::{expected, keys, text};
use super::constants::{IDENTITY_COMMAND, NAMESPACE};
use crate::crypt::Provider;
use crate::lua::Config;

pub fn set(lua: &Lua, value: Value) -> mlua::Result<()> {
    if surface::inert(
        lua,
        &format!("{NAMESPACE}.{IDENTITY_COMMAND}"),
        Surface::Config,
    ) {
        return Ok(());
    }

    let provider = provider(&value)?;
    Config::building(lua)?.set_crypt_identity_command(provider);
    Ok(())
}

fn provider(value: &Value) -> mlua::Result<Provider> {
    let kind = "a command line or a list of a program and its arguments";
    match value {
        Value::String(_) => line(value, kind),
        Value::Table(_) => keys(NAMESPACE, value, IDENTITY_COMMAND)
            .map(Provider::Program)
            .map_err(|_| expected(NAMESPACE, IDENTITY_COMMAND, kind)),
        _ => Err(expected(NAMESPACE, IDENTITY_COMMAND, kind)),
    }
}

fn line(value: &Value, kind: &str) -> mlua::Result<Provider> {
    let line = text(NAMESPACE, value, IDENTITY_COMMAND)?;
    if line.trim().is_empty() {
        return Err(expected(NAMESPACE, IDENTITY_COMMAND, kind));
    }

    Ok(Provider::Line(line))
}

#[cfg(test)]
mod tests {
    use crate::crypt::Provider;
    use crate::lua::from_source;

    fn program(words: &[&str]) -> Provider {
        Provider::Program(words.iter().map(|word| word.to_string()).collect())
    }

    #[test]
    fn defaults_to_no_provider() {
        let config = from_source("local unused = 1").unwrap();

        assert_eq!(config.crypt_identity_command(), None);
    }

    #[test]
    fn takes_a_command_line() {
        let config = from_source(r#"ld.crypt.identity_command("pass show age/key")"#).unwrap();

        assert_eq!(
            config.crypt_identity_command(),
            Some(&Provider::Line("pass show age/key".to_string()))
        );
    }

    #[test]
    fn takes_a_program_and_its_arguments() {
        let config =
            from_source(r#"ld.crypt.identity_command({ "op", "read", "op://vault/age/key" })"#)
                .unwrap();

        assert_eq!(
            config.crypt_identity_command(),
            Some(&program(&["op", "read", "op://vault/age/key"]))
        );
    }

    #[test]
    fn rejects_an_empty_command() {
        for source in [
            r#"ld.crypt.identity_command("  ")"#,
            "ld.crypt.identity_command({})",
            "ld.crypt.identity_command(42)",
        ] {
            let err = format!("{:#}", from_source(source).unwrap_err());

            assert!(
                err.contains("`ld.crypt.identity_command` takes a command line"),
                "{source} reported {err}"
            );
        }
    }
}
