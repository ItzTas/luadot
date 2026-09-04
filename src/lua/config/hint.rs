use mlua::{IntoLua, Lua, Value};

use super::constants::{DEFAULT, NAME};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hint {
    name: &'static str,
    default: &'static str,
}

impl Hint {
    pub fn new(name: &'static str, default: &'static str) -> Self {
        Self { name, default }
    }

    pub fn default(self) -> &'static str {
        self.default
    }
}

impl IntoLua for Hint {
    fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
        let hint = lua.create_table()?;
        hint.set(NAME, self.name)?;
        hint.set(DEFAULT, self.default)?;

        Ok(Value::Table(hint))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_hint_carries_its_name_and_line() {
        let lua = Lua::new();
        lua.globals()
            .set("subject", Hint::new("differs", "(use \"luadot diff\")"))
            .unwrap();

        assert_eq!(
            lua.load("return subject.name").eval::<String>().unwrap(),
            "differs"
        );
        assert_eq!(
            lua.load("return subject.default").eval::<String>().unwrap(),
            "(use \"luadot diff\")"
        );
    }
}
