use std::os::raw::c_int;

use mlua::{Lua, Table, lua_State};

use super::constants::{LPEG_MODULE, PACKAGE, RE_MODULE};

const PRELOAD: &str = "preload";

const RE_CHUNK: &str = "@re.lua";

const RE_SOURCE: &str = include_str!(env!("LPEG_RE_PATH"));

unsafe extern "C-unwind" {
    fn luaopen_lpeg(state: *mut lua_State) -> c_int;
}

pub fn preload(lua: &Lua) -> mlua::Result<()> {
    let package: Table = lua.globals().get(PACKAGE)?;
    let preload: Table = package.get(PRELOAD)?;

    let open_lpeg = unsafe { lua.create_c_function(luaopen_lpeg)? };
    preload.set(LPEG_MODULE, open_lpeg)?;

    let open_re = lua.load(RE_SOURCE).set_name(RE_CHUNK).into_function()?;
    preload.set(RE_MODULE, open_re)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua::runtime::runtime;

    #[test]
    fn lpeg_matches_a_pattern() {
        let matched: i64 = runtime()
            .unwrap()
            .load("local lpeg = require(\"lpeg\") return lpeg.match(lpeg.P(\"ab\"), \"abc\")")
            .eval()
            .unwrap();

        assert_eq!(matched, 3);
    }
}
