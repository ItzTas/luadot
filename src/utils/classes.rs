use anyhow::Result;

use crate::lua::Class;
use crate::output::choose;
use crate::state::{self, State};

pub fn ask(command: &str, class: &Class, current: Option<&str>) -> Result<String> {
    choose(
        command,
        class.name(),
        &class.question(),
        class.choices(),
        current.or_else(|| class.default()),
        &skip(class.name()),
    )
}

pub fn ask_missing(command: &str, declared: &[Class]) -> Result<usize> {
    let mut state = state::load()?;
    let missing = missing(declared, &state);
    if missing.is_empty() {
        return Ok(0);
    }

    for class in &missing {
        let value = ask(command, class, None)?;
        state.set_class(class.name(), &value);
    }
    state::save(&state)?;

    Ok(missing.len())
}

fn missing<'a>(declared: &'a [Class], state: &State) -> Vec<&'a Class> {
    declared
        .iter()
        .filter(|class| state.class(class.name()).is_none())
        .collect()
}

fn skip(name: &str) -> String {
    format!("it with `luadot class set {name} <value>`")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn class(name: &str) -> Class {
        Class::new(name.to_string(), None, Vec::new(), None)
    }

    #[test]
    fn only_the_classes_nobody_answered_are_missing() {
        let declared = [class("form-factor"), class("email")];
        let mut state = State::default();
        state.set_class("form-factor", "laptop");

        let missing = missing(&declared, &state);

        assert_eq!(missing.len(), 1);
        assert_eq!(missing[0].name(), "email");
    }

    #[test]
    fn asking_without_a_terminal_reports_the_way_out() {
        let err = ask("class", &class("form-factor"), None)
            .unwrap_err()
            .to_string();

        assert!(err.contains("cannot ask for `form-factor` without a terminal"));
        assert!(err.contains("luadot class set form-factor <value>"));
    }
}
