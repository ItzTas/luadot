pub const DEFINITIONS: &str = include_str!("../../../meta/ld.lua");

pub const DEFINITIONS_DIR: &str = "meta";

pub const DEFINITIONS_FILE: &str = "ld.lua";

pub const LUARC_FILE: &str = ".luarc.json";

pub const SCHEMA_KEY: &str = "$schema";

pub const SCHEMA: &str =
    "https://raw.githubusercontent.com/LuaLS/vscode-lua/master/setting/schema.json";

pub const VERSION_KEY: &str = "runtime.version";

pub const VERSION: &str = "Lua 5.4";

pub const PATH_KEY: &str = "runtime.path";

pub const PATHS: [&str; 4] = ["?.lua", "?/init.lua", "lua/?.lua", "lua/?/init.lua"];

pub const LIBRARY_KEY: &str = "workspace.library";

pub const TILDE: &str = "~";

pub const NOT_AN_OBJECT: &str = "the settings are not a JSON object";
