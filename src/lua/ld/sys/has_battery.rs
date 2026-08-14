use std::fs;
use std::path::Path;

use mlua::{Function, Lua};

use super::constants::{BATTERY_TYPE, DEVICE_SCOPE, POWER_SUPPLY_DIR, SCOPE_FILE, TYPE_FILE};

pub fn function(lua: &Lua) -> mlua::Result<Function> {
    lua.create_function(|_, ()| Ok(has_battery(Path::new(POWER_SUPPLY_DIR))))
}

fn has_battery(supplies: &Path) -> bool {
    fs::read_dir(supplies)
        .into_iter()
        .flatten()
        .flatten()
        .any(|entry| is_battery(&entry.path()))
}

fn is_battery(supply: &Path) -> bool {
    if read(&supply.join(TYPE_FILE)).as_deref() != Some(BATTERY_TYPE) {
        return false;
    }

    read(&supply.join(SCOPE_FILE)).is_none_or(|scope| scope != DEVICE_SCOPE)
}

fn read(path: &Path) -> Option<String> {
    Some(fs::read_to_string(path).ok()?.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn supplies(entries: &[(&str, &str, Option<&str>)]) -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        for (name, kind, scope) in entries {
            let supply = dir.path().join(name);
            fs::create_dir_all(&supply).unwrap();
            fs::write(supply.join(TYPE_FILE), format!("{kind}\n")).unwrap();
            if let Some(scope) = scope {
                fs::write(supply.join(SCOPE_FILE), format!("{scope}\n")).unwrap();
            }
        }

        dir
    }

    #[test]
    fn a_laptop_carries_a_battery() {
        let dir = supplies(&[("AC", "Mains", None), ("BAT0", "Battery", Some("System"))]);

        assert!(has_battery(dir.path()));
    }

    #[test]
    fn a_desktop_carries_none() {
        let dir = supplies(&[("AC", "Mains", None)]);

        assert!(!has_battery(dir.path()));
        assert!(!has_battery(Path::new("/nonexistent/power_supply")));
    }

    #[test]
    fn the_battery_of_a_peripheral_is_not_the_machines() {
        let dir = supplies(&[("hidpp_battery_0", "Battery", Some("Device"))]);

        assert!(!has_battery(dir.path()));
    }
}
