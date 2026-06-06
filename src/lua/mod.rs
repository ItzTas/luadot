#![allow(dead_code)]

use mlua::Lua;

pub fn runtime() -> mlua::Result<Lua> {
    Ok(Lua::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_evaluates_expression() {
        let lua = runtime().unwrap();
        let value: i64 = lua.load("return 1 + 2").eval().unwrap();
        assert_eq!(value, 3);
    }
}
