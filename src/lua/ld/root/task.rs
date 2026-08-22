use mlua::{Function, Lua, Table, Value};

use super::super::constants::API;
use super::super::parse::{external, known};
use super::super::surface::{self, Surface};
use super::constants::{ABOUT, BUILTINS, RUN, TASK, TASK_KEYS};
use crate::lua::{Call, Config, Task};

pub fn function(lua: &Lua) -> mlua::Result<Function> {
    lua.create_function(|lua, (value, spec): (Value, Table)| {
        if surface::inert(lua, TASK, Surface::Config) {
            return Ok(());
        }

        let name = name(&value)?;
        known(TASK, &spec, &TASK_KEYS)?;
        let task = task(&spec)?;

        if !Config::building(lua, |config| config.add_task(name.clone(), task))? {
            return Err(external(format!(
                "`{API}.{TASK}`: `{name}` is already registered"
            )));
        }

        Ok(())
    })
}

fn name(value: &Value) -> mlua::Result<String> {
    let Value::String(name) = value else {
        return Err(external(format!(
            "`{API}.{TASK}` takes a name, got {}",
            value.type_name()
        )));
    };
    let name = name.to_str()?.to_string();

    if name.is_empty() || name.contains(char::is_whitespace) || name.starts_with('-') {
        return Err(external(format!(
            "`{API}.{TASK}` takes a name without spaces and not starting with `-`, got `{name}`"
        )));
    }
    if BUILTINS.contains(&name.as_str()) {
        return Err(external(format!(
            "`{API}.{TASK}`: `{name}` is a command of luadot's own"
        )));
    }

    Ok(name)
}

fn task(spec: &Table) -> mlua::Result<Task> {
    let about = match spec.get::<Value>(ABOUT)? {
        Value::Nil => None,
        Value::String(text) => Some(text.to_str()?.to_string()),
        other => {
            return Err(external(format!(
                "`{API}.{TASK}`: `{ABOUT}` takes a string, got {}",
                other.type_name()
            )));
        }
    };
    let run = match spec.get::<Value>(RUN)? {
        Value::Function(function) => Call::new(function),
        other => {
            return Err(external(format!(
                "`{API}.{TASK}`: `{RUN}` takes a function, got {}",
                other.type_name()
            )));
        }
    };

    Ok(Task::new(about, run))
}

#[cfg(test)]
mod tests {
    use crate::lua::from_source;

    fn error(source: &str) -> String {
        format!("{:#}", from_source(source).unwrap_err())
    }

    #[test]
    fn a_task_is_kept_under_its_name_with_its_function_callable() {
        let config = from_source(
            r#"
            ld.task("plug", {
              about = "Manage plugins",
              run = function(argv) return "plug " .. table.concat(argv, " ") end,
            })
            "#,
        )
        .unwrap();

        let task = config.task("plug").unwrap();
        assert_eq!(task.about(), Some("Manage plugins"));
        assert_eq!(
            task.run("task `plug`", vec!["sync".to_string()]).unwrap(),
            Some("plug sync".to_string())
        );
        assert_eq!(config.tasks().count(), 1);
    }

    #[test]
    fn the_name_of_a_command_luadot_has_is_refused() {
        let err = error(r#"ld.task("apply", { run = function() end })"#);

        assert!(err.contains("`ld.task`: `apply` is a command of luadot's own"));
    }

    #[test]
    fn a_name_registered_twice_is_refused() {
        let err = error(
            r#"
            ld.task("plug", { run = function() end })
            ld.task("plug", { run = function() end })
            "#,
        );

        assert!(err.contains("`ld.task`: `plug` is already registered"));
    }

    #[test]
    fn a_task_needs_a_function_to_run_and_a_name_the_command_line_takes() {
        assert!(
            error(r#"ld.task("plug", { about = "x" })"#)
                .contains("`ld.task`: `run` takes a function, got nil")
        );
        assert!(
            error(r#"ld.task("plug", { run = function() end, runs = 1 })"#)
                .contains("`ld.task`: unknown key `runs`")
        );
        assert!(
            error(r#"ld.task("--plug", { run = function() end })"#)
                .contains("takes a name without spaces and not starting with `-`")
        );
    }
}
