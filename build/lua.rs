use std::path::PathBuf;

pub fn headers() -> PathBuf {
    lua_src::Build::new()
        .build(lua_src::Lua54)
        .include_dir()
        .to_path_buf()
}
