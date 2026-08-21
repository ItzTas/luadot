use anyhow::{Result, bail};
use clap::{Args, Subcommand};

use crate::lua::{self, Class, Config};
use crate::output;
use crate::state;
use crate::utils;

use super::super::constants::{UNDECLARED, UNSET};

#[derive(Debug, Args)]
pub struct ClassArgs {
    #[command(subcommand)]
    pub action: Option<ClassAction>,
}

#[derive(Debug, Subcommand)]
pub enum ClassAction {
    #[command(about = "List every declared class with the answer of this machine")]
    List,
    #[command(about = "Answer a class, or ask for every one still unanswered")]
    Set {
        #[arg(value_name = "NAME")]
        name: Option<String>,
        #[arg(value_name = "VALUE")]
        value: Vec<String>,
    },
    #[command(about = "Forget the answer of a class")]
    Unset {
        #[arg(value_name = "NAME")]
        name: String,
    },
    #[command(about = "Print the answer alone, for a script to read")]
    Get {
        #[arg(value_name = "NAME")]
        name: String,
    },
}

pub fn class_cmd(args: ClassArgs) -> Result<()> {
    match args.action.unwrap_or(ClassAction::List) {
        ClassAction::List => list(),
        ClassAction::Set { name, value } => set(name, value),
        ClassAction::Unset { name } => unset(&name),
        ClassAction::Get { name } => get(&name),
    }
}

fn list() -> Result<()> {
    let config = lua::load_config()?;
    let config = utils::configured("class", &config)?;
    let state = state::load()?;

    if config.classes().is_empty() && state.classes().is_empty() {
        output::note(format!(
            "no class declared; declare one with `ld.class` in {}",
            lua::config_path()?.display()
        ));
        return Ok(());
    }

    for class in config.classes() {
        output::field(class.name(), state.class(class.name()).unwrap_or(UNSET));
    }
    for (name, value) in state
        .classes()
        .iter()
        .filter(|(name, _)| config.class(name).is_none())
    {
        output::field(name, format!("{value}  {UNDECLARED}"));
    }

    Ok(())
}

fn set(name: Option<String>, values: Vec<String>) -> Result<()> {
    let config = lua::load_config()?;
    let config = utils::configured("class", &config)?;
    let Some(name) = name else {
        return set_missing(&config);
    };

    let class = declared(&config, &name)?;
    let mut state = state::load()?;
    let value = match values.is_empty() {
        false => checked(class, values.join(" "))?,
        true => utils::ask("class", class, state.class(&name))?,
    };

    state.set_class(&name, &value);
    state::save(&state)?;
    output::note(format!("{name} is {value}"));

    Ok(())
}

fn set_missing(config: &Config) -> Result<()> {
    if config.classes().is_empty() {
        bail!("class: no class declared; declare one with `ld.class` in config.lua");
    }

    let asked = utils::ask_missing("class", config.classes())?;
    if asked == 0 {
        output::note("every declared class is already set");
        return Ok(());
    }
    output::note(format!("set {asked} class(es)"));

    Ok(())
}

fn unset(name: &str) -> Result<()> {
    let mut state = state::load()?;

    if !state.unset_class(name) {
        output::note(format!("{name} is not set"));
        return Ok(());
    }

    state::save(&state)?;
    output::note(format!("{name} is back to unset"));

    Ok(())
}

fn get(name: &str) -> Result<()> {
    let state = state::load()?;

    let Some(value) = state.class(name) else {
        bail!("class: {name} is not set");
    };
    output::line(value);

    Ok(())
}

fn declared<'a>(config: &'a Config, name: &str) -> Result<&'a Class> {
    let Some(class) = config.class(name) else {
        bail!(
            "class: no class named `{name}` (declared: {})",
            names(config)
        );
    };

    Ok(class)
}

fn names(config: &Config) -> String {
    if config.classes().is_empty() {
        return "none".to_string();
    }

    config
        .classes()
        .iter()
        .map(Class::name)
        .collect::<Vec<_>>()
        .join(", ")
}

fn checked(class: &Class, value: String) -> Result<String> {
    if class.choices().is_empty() || class.choices().contains(&value) {
        return Ok(value);
    }

    bail!(
        "class: `{value}` is not one of the choices of `{}` (available: {})",
        class.name(),
        class.choices().join(", ")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn class() -> Class {
        Class::new(
            "form-factor".to_string(),
            None,
            vec!["desktop".to_string(), "laptop".to_string()],
            None,
        )
    }

    #[test]
    fn a_value_of_the_choices_is_taken() {
        assert_eq!(checked(&class(), "laptop".to_string()).unwrap(), "laptop");
    }

    #[test]
    fn a_value_outside_the_choices_is_reported() {
        let err = checked(&class(), "tablet".to_string())
            .unwrap_err()
            .to_string();

        assert!(err.contains("`tablet` is not one of the choices of `form-factor`"));
        assert!(err.contains("available: desktop, laptop"));
    }

    #[test]
    fn an_undeclared_class_lists_the_declared_ones() {
        let mut config = Config::default();
        config.add_class(class());

        let err = declared(&config, "editor").unwrap_err().to_string();

        assert!(err.contains("class: no class named `editor`"));
        assert!(err.contains("declared: form-factor"));
    }
}
