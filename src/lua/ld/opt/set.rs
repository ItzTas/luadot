use mlua::{Lua, Value};

use super::super::constants::{API, CONFLICT_POLICIES, LINK_MODES};
use super::super::parse::external;
use super::super::value::{choice, count, flag, path, span};
use super::constants::{
    AUTOCOMMIT, AUTOPUSH, BACKUP, BACKUP_AGE, BACKUP_DIR, BACKUP_KEEP, CONFLICT, DIRECTORY_KIND,
    HINTS, LFS, LINK, NAMESPACE, PASSPHRASE_WARN, PKG_WARN, REPO_DIR, SPAN_KIND,
};
use crate::lua::Config;

pub fn autocommit(lua: &Lua, value: Value) -> mlua::Result<()> {
    let enabled = flag(NAMESPACE, &value, AUTOCOMMIT)?;
    Config::building(lua, |config| config.set_autocommit(enabled))
}

pub fn autopush(lua: &Lua, value: Value) -> mlua::Result<()> {
    let enabled = flag(NAMESPACE, &value, AUTOPUSH)?;
    Config::building(lua, |config| config.set_autopush(enabled))
}

pub fn backup(lua: &Lua, value: Value) -> mlua::Result<()> {
    let enabled = flag(NAMESPACE, &value, BACKUP)?;
    Config::building(lua, |config| config.set_backup(enabled))
}

pub fn backup_age(lua: &Lua, value: Value) -> mlua::Result<()> {
    let age = span(NAMESPACE, &value, BACKUP_AGE, SPAN_KIND)?;
    if age == 0 {
        return Err(off(BACKUP_AGE, "one second or more"));
    }

    Config::building(lua, |config| config.set_backup_age(age))
}

pub fn backup_dir(lua: &Lua, value: Value) -> mlua::Result<()> {
    let dir = path(NAMESPACE, &value, BACKUP_DIR, DIRECTORY_KIND)?;
    Config::building(lua, |config| config.set_backup_dir(dir))
}

pub fn backup_keep(lua: &Lua, value: Value) -> mlua::Result<()> {
    let keep = count(NAMESPACE, &value, BACKUP_KEEP)?;
    if keep == 0 {
        return Err(off(BACKUP_KEEP, "one or more"));
    }

    Config::building(lua, |config| config.set_backup_keep(keep))
}

pub fn conflict(lua: &Lua, value: Value) -> mlua::Result<()> {
    let policy = choice(
        NAMESPACE,
        &value,
        CONFLICT,
        &CONFLICT_POLICIES,
        "conflict policy",
    )?;
    Config::building(lua, |config| config.set_conflict(policy))
}

pub fn hints(lua: &Lua, value: Value) -> mlua::Result<()> {
    let enabled = flag(NAMESPACE, &value, HINTS)?;
    Config::building(lua, |config| config.set_hints(enabled))
}

pub fn lfs(lua: &Lua, value: Value) -> mlua::Result<()> {
    let enabled = flag(NAMESPACE, &value, LFS)?;
    Config::building(lua, |config| config.set_lfs(enabled))
}

pub fn link(lua: &Lua, value: Value) -> mlua::Result<()> {
    let mode = choice(NAMESPACE, &value, LINK, &LINK_MODES, "link mode")?;
    Config::building(lua, |config| config.set_link(mode))
}

pub fn passphrase_warn(lua: &Lua, value: Value) -> mlua::Result<()> {
    let enabled = flag(NAMESPACE, &value, PASSPHRASE_WARN)?;
    Config::building(lua, |config| config.set_passphrase_warn(enabled))
}

pub fn pkg_warn(lua: &Lua, value: Value) -> mlua::Result<()> {
    let enabled = flag(NAMESPACE, &value, PKG_WARN)?;
    Config::building(lua, |config| config.set_pkg_warn(enabled))
}

pub fn repo_dir(lua: &Lua, value: Value) -> mlua::Result<()> {
    let dir = path(NAMESPACE, &value, REPO_DIR, DIRECTORY_KIND)?;
    Config::building(lua, |config| config.set_repo_dir(dir))
}

fn off(option: &str, takes: &str) -> mlua::Error {
    external(format!(
        "`{API}.{NAMESPACE}.{option}` takes {takes}; `{API}.{NAMESPACE}.{BACKUP}(false)` is how backups are turned off"
    ))
}
