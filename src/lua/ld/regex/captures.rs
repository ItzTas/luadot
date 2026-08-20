use mlua::{Lua, Value, Variadic};
use regex::Captures;

pub fn owned(captures: &Captures) -> Vec<Option<String>> {
    captures
        .iter()
        .map(|group| group.map(|group| group.as_str().to_string()))
        .collect()
}

pub fn values(lua: &Lua, groups: &[Option<String>]) -> mlua::Result<Variadic<Value>> {
    groups
        .iter()
        .map(|group| match group {
            Some(text) => lua.create_string(text).map(Value::String),
            None => Ok(Value::Nil),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use regex::Regex;

    use super::*;
    use crate::lua::runtime::runtime;

    #[test]
    fn a_group_that_did_not_participate_is_nil() {
        let lua = runtime().unwrap();
        let regex = Regex::new(r"(a)|(b)").unwrap();
        let captures = regex.captures("a").unwrap();

        let values = values(&lua, &owned(&captures)).unwrap();

        assert_eq!(values.len(), 3);
        assert!(matches!(values[2], Value::Nil));
    }
}
