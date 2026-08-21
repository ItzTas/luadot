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

#[cfg(feature = "meta")]
pub const JSON_FLAG: &str = "--json";

#[cfg(feature = "meta")]
pub const USAGE: &str = "luadot-meta: usage: luadot-meta [--json]";

#[cfg(feature = "meta")]
pub const JSON_FAILED: &str = "luadot-meta: failed to serialize the description";

#[cfg(feature = "meta")]
pub const HEADER: &str = "---@meta";

#[cfg(feature = "meta")]
pub const COMMENT: &str = "---";

#[cfg(feature = "meta")]
pub const ALIAS: &str = "---@alias";

#[cfg(feature = "meta")]
pub const VARIANT: &str = "---|";

#[cfg(feature = "meta")]
pub const CLASS: &str = "---@class";

#[cfg(feature = "meta")]
pub const FIELD: &str = "---@field";

#[cfg(feature = "meta")]
pub const OVERLOAD: &str = "---@overload";

#[cfg(feature = "meta")]
pub const PARAM: &str = "---@param";

#[cfg(feature = "meta")]
pub const RETURN: &str = "---@return";

#[cfg(feature = "meta")]
pub const FUNCTION: &str = "function";

#[cfg(feature = "meta")]
pub const END: &str = "end";

#[cfg(feature = "meta")]
pub const FUN: &str = "fun";

#[cfg(feature = "meta")]
pub const EMPTY_TABLE: &str = "= {}";

#[cfg(feature = "meta")]
pub const ELLIPSIS: &str = "...";

#[cfg(feature = "meta")]
pub const OPTIONAL: &str = "?";

#[cfg(feature = "meta")]
pub const UNNAMED: &str = "_";

#[cfg(feature = "meta")]
pub const ARRAY: &str = "[]";

#[cfg(feature = "meta")]
pub const MAP: &str = "table";
