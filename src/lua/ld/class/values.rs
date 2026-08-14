use mlua::Lua;

use crate::state::Classes;

pub fn install(lua: &Lua, classes: &Classes) {
    lua.set_app_data(classes.clone());
}

pub fn current(lua: &Lua) -> Classes {
    lua.app_data_ref::<Classes>()
        .map(|classes| classes.clone())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lua::runtime::runtime;

    #[test]
    fn a_runtime_without_values_answers_with_none() {
        let lua = runtime().unwrap();

        assert!(current(&lua).is_empty());
    }

    #[test]
    fn the_installed_values_are_the_current_ones() {
        let lua = runtime().unwrap();
        let mut classes = Classes::default();
        classes.set("form-factor", "laptop");

        install(&lua, &classes);

        assert_eq!(current(&lua).get("form-factor"), Some("laptop"));
    }
}
