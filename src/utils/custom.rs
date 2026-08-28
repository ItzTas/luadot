use anyhow::Result;

use crate::lua::{Command, Config, HINTS, Hint};
use crate::output;

pub fn customized(command: &str, call: &str, key: &str) -> String {
    format!("{command}: `{call}`: `{key}`")
}

pub fn said(shown: Option<String>) {
    let Some(text) = shown else {
        return;
    };

    output::line(text);
}

pub fn hint(config: &Config, command: Command, hint: Hint) -> Result<()> {
    let Some(custom) = config.command_hints(command) else {
        if config.hints() {
            output::hint(hint.default());
        }

        return Ok(());
    };

    said(custom.shown(&customized(command.name(), &command.call(), HINTS), hint)?);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lua::from_source;

    #[test]
    fn a_failure_names_command_call_key() {
        let config =
            from_source(r#"ld.on.status({ hints = function() error("broken") end })"#).unwrap();

        let err = format!(
            "{:#}",
            hint(&config, Command::Status, Hint::new("differs", "(use it)")).unwrap_err()
        );

        assert!(err.contains("status: `ld.on.status`: `hints` failed"));
        assert!(err.contains("broken"));
    }
}
