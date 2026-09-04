use anyhow::{Context, Result, bail};
use mlua::{Function, IntoLuaMulti, Value};

#[derive(Debug, Clone)]
pub struct Call(Function);

#[derive(Debug, Clone)]
pub enum Custom {
    Silent,
    Text(String),
    Call(Call),
}

impl Call {
    pub fn new(function: Function) -> Self {
        Self(function)
    }

    pub fn run(&self, what: &str, argument: impl IntoLuaMulti) -> Result<Option<String>> {
        let answer = self
            .0
            .call::<Value>(argument)
            .with_context(|| format!("{what} failed"))?;

        match answer {
            Value::Nil | Value::Boolean(false) => Ok(None),
            Value::String(text) => Ok(Some(text.to_str()?.to_string())),
            other => bail!(
                "{what} returned {}; a string or nothing is expected",
                other.type_name()
            ),
        }
    }
}

impl Custom {
    pub fn shown(&self, what: &str, argument: impl IntoLuaMulti) -> Result<Option<String>> {
        match self {
            Self::Silent => Ok(None),
            Self::Text(text) => Ok(Some(text.clone())),
            Self::Call(call) => call.run(what, argument),
        }
    }
}

#[cfg(test)]
mod tests {
    use mlua::Lua;

    use super::*;

    fn call(lua: &Lua, source: &str) -> Call {
        Call::new(lua.load(source).eval().unwrap())
    }

    #[test]
    fn a_call_answers_with_its_string() {
        let lua = Lua::new();
        let call = call(
            &lua,
            r#"return function(count) return "seen " .. count end"#,
        );

        assert_eq!(call.run("a hook", 3).unwrap(), Some("seen 3".to_string()));
    }

    #[test]
    fn any_other_return_is_reported() {
        let lua = Lua::new();
        let call = call(&lua, "return function() return 1 end");

        let err = format!("{:#}", call.run("`ld.on.diff`: `summary`", ()).unwrap_err());

        assert!(err.contains("`ld.on.diff`: `summary` returned integer"));
        assert!(err.contains("a string or nothing is expected"));
    }
}
