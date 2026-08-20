use mlua::{Function, Lua, Table, Value};

use super::super::constants::API;
use super::super::parse::external;
use super::super::surface::{self, Surface};
use super::constants::{CHOICES, DEFAULT, NAME, NAMESPACE, PROMPT};
use crate::lua::{Class, Config};

pub fn function(lua: &Lua) -> mlua::Result<Function> {
    lua.create_function(|lua, (_, value): (Table, Value)| {
        if surface::inert(lua, NAMESPACE, Surface::Config) {
            return Ok(());
        }

        let class = class(&value)?;
        Config::building(lua)?.add_class(class);
        Ok(())
    })
}

fn class(value: &Value) -> mlua::Result<Class> {
    let Value::Table(entry) = value else {
        return Err(expected(format!("a table, got {}", value.type_name())));
    };

    let name = name(entry)?;
    let prompt: Option<String> = entry.get(PROMPT)?;
    let choices = choices(entry)?;
    let default = default(entry, &choices)?;

    Ok(Class::new(name, prompt, choices, default))
}

fn name(entry: &Table) -> mlua::Result<String> {
    let name: Option<String> = entry.get(NAME)?;
    let name = name.ok_or_else(|| expected(format!("a `{NAME}`")))?;

    if name.trim().is_empty() {
        return Err(expected(format!("a `{NAME}` that is not empty")));
    }
    if name.split_whitespace().count() > 1 || name.trim() != name {
        return Err(external(format!(
            "`{API}.{NAMESPACE}` name `{name}` cannot hold spaces"
        )));
    }

    Ok(name)
}

fn choices(entry: &Table) -> mlua::Result<Vec<String>> {
    match entry.get::<Value>(CHOICES)? {
        Value::Nil => Ok(Vec::new()),
        Value::String(one) => Ok(vec![one.to_str()?.to_string()]),
        Value::Table(list) => from_list(&list),
        other => Err(expected(format!(
            "`{CHOICES}` as a string or a table of strings, got {}",
            other.type_name()
        ))),
    }
}

fn from_list(list: &Table) -> mlua::Result<Vec<String>> {
    list.clone()
        .sequence_values::<String>()
        .enumerate()
        .map(|(index, choice)| {
            let choice = choice.map_err(|_| {
                external(format!(
                    "`{API}.{NAMESPACE}` {CHOICES} entry {} is not a string",
                    index + 1
                ))
            })?;
            if choice.is_empty() {
                return Err(external(format!(
                    "`{API}.{NAMESPACE}` {CHOICES} entry {} is empty",
                    index + 1
                )));
            }
            Ok(choice)
        })
        .collect()
}

fn default(entry: &Table, choices: &[String]) -> mlua::Result<Option<String>> {
    let default: Option<String> = entry.get(DEFAULT)?;
    let Some(default) = default else {
        return Ok(None);
    };

    if !choices.is_empty() && !choices.contains(&default) {
        return Err(external(format!(
            "`{API}.{NAMESPACE}` {DEFAULT} `{default}` is not one of its {CHOICES} ({})",
            choices.join(", ")
        )));
    }

    Ok(Some(default))
}

fn expected(what: String) -> mlua::Error {
    external(format!("`{API}.{NAMESPACE}` takes {what}"))
}

#[cfg(test)]
mod tests {
    use crate::lua::{Config, from_source};

    fn configure(source: &str) -> Config {
        from_source(source).unwrap()
    }

    fn error(source: &str) -> String {
        format!("{:#}", from_source(source).unwrap_err())
    }

    #[test]
    fn declares_a_class_with_its_prompt_and_choices() {
        let config = configure(
            r#"
            ld.class({
              name = "form-factor",
              prompt = "Is this machine a desktop or a laptop?",
              choices = { "desktop", "laptop" },
              default = "laptop",
            })
            "#,
        );

        let class = config.class("form-factor").unwrap();
        assert_eq!(class.question(), "Is this machine a desktop or a laptop?");
        assert_eq!(class.choices(), ["desktop", "laptop"]);
        assert_eq!(class.default(), Some("laptop"));
    }

    #[test]
    fn a_class_needs_nothing_but_a_name() {
        let config = configure(r#"ld.class({ name = "email" })"#);

        let class = config.class("email").unwrap();
        assert_eq!(class.question(), "define the class `email`");
        assert!(class.choices().is_empty());
        assert_eq!(class.default(), None);
    }

    #[test]
    fn a_single_choice_is_taken_as_a_list_of_one() {
        let config = configure(r#"ld.class({ name = "shell", choices = "zsh" })"#);

        assert_eq!(config.class("shell").unwrap().choices(), ["zsh"]);
    }

    #[test]
    fn rejects_a_declaration_without_a_name() {
        assert!(error(r#"ld.class({ prompt = "which one?" })"#).contains("takes a `name`"));
    }

    #[test]
    fn rejects_an_argument_that_is_not_a_table() {
        assert!(error(r#"ld.class("form-factor")"#).contains("takes a table, got string"));
    }

    #[test]
    fn rejects_a_name_holding_spaces() {
        let err = error(r#"ld.class({ name = "form factor" })"#);

        assert!(err.contains("name `form factor` cannot hold spaces"));
    }

    #[test]
    fn rejects_choices_of_the_wrong_type() {
        let err = error(r#"ld.class({ name = "shell", choices = 42 })"#);

        assert!(err.contains("takes `choices` as a string or a table of strings"));
    }

    #[test]
    fn rejects_an_empty_choice() {
        let err = error(r#"ld.class({ name = "shell", choices = { "zsh", "" } })"#);

        assert!(err.contains("choices entry 2 is empty"));
    }

    #[test]
    fn rejects_a_default_outside_the_choices() {
        let err =
            error(r#"ld.class({ name = "shell", choices = { "zsh", "fish" }, default = "bash" })"#);

        assert!(err.contains("default `bash` is not one of its choices (zsh, fish)"));
    }
}
