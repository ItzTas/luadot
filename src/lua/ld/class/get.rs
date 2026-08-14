use mlua::{Function, Lua, Value};

use super::super::constants::API;
use super::super::parse::external;
use super::constants::{GET, NAMESPACE};
use crate::state::Classes;

pub fn function(lua: &Lua, classes: &Classes) -> mlua::Result<Function> {
    let classes = classes.clone();

    lua.create_function(move |_, value: Value| {
        let name = name(&value)?;

        Ok(classes.get(&name).map(str::to_string))
    })
}

fn name(value: &Value) -> mlua::Result<String> {
    match value {
        Value::String(name) => Ok(name.to_str()?.to_string()),
        other => Err(external(format!(
            "`{API}.{NAMESPACE}.{GET}` takes a class name, got {}",
            other.type_name()
        ))),
    }
}

#[cfg(test)]
mod tests {
    use crate::lua::from_classes;
    use crate::state::Classes;

    fn classes() -> Classes {
        let mut classes = Classes::default();
        classes.set("form-factor", "laptop");
        classes
    }

    #[test]
    fn reads_the_value_the_machine_answered() {
        assert!(
            from_classes(
                r#"assert(ld.class.get("form-factor") == "laptop", "wrong value")"#,
                &classes(),
            )
            .is_ok()
        );
    }

    #[test]
    fn a_class_nobody_answered_yields_nil() {
        assert!(
            from_classes(
                r#"assert(ld.class.get("editor") == nil, "editor is set")"#,
                &classes(),
            )
            .is_ok()
        );
    }

    #[test]
    fn the_value_drives_the_configuration() {
        let config = from_classes(
            r#"
            if ld.class.get("form-factor") == "laptop" then
              ld.opt.link("symbolic")
            end
            "#,
            &classes(),
        )
        .unwrap();

        assert_eq!(
            config.link_mode(std::path::Path::new(".bashrc")),
            crate::files::LinkMode::Symbolic
        );
    }

    #[test]
    fn rejects_an_argument_that_is_not_a_name() {
        let err = format!(
            "{:#}",
            from_classes("ld.class.get(42)", &classes()).unwrap_err()
        );

        assert!(err.contains("`ld.class.get` takes a class name, got integer"));
    }
}
