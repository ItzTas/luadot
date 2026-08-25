use std::path::{Path, PathBuf};

use serde::de::Error;
use serde_json::{Map, Value, json};

use super::constants::{
    DEFINITIONS_DIR, LIBRARY_KEY, NOT_AN_OBJECT, PATH_KEY, PATHS, SCHEMA, SCHEMA_KEY, TILDE,
    VERSION, VERSION_KEY,
};

pub fn merged(existing: Option<&str>, libraries: &[String]) -> serde_json::Result<String> {
    let mut settings = match existing {
        Some(text) => parsed(text)?,
        None => Map::new(),
    };
    for (key, value) in wanted(libraries) {
        merge(&mut settings, key, value);
    }

    let mut text = serde_json::to_string_pretty(&Value::Object(settings))?;
    text.push('\n');

    Ok(text)
}

pub fn libraries(home: &Path, data: &Path, registered: &[PathBuf]) -> Vec<String> {
    std::iter::once(data.join(DEFINITIONS_DIR))
        .chain(registered.iter().cloned())
        .map(|dir| shortened(home, &dir))
        .collect()
}

fn shortened(home: &Path, dir: &Path) -> String {
    match dir.strip_prefix(home) {
        Ok(relative) => Path::new(TILDE).join(relative).display().to_string(),
        Err(_) => dir.display().to_string(),
    }
}

fn parsed(text: &str) -> serde_json::Result<Map<String, Value>> {
    match serde_json::from_str(text)? {
        Value::Object(settings) => Ok(settings),
        _ => Err(serde_json::Error::custom(NOT_AN_OBJECT)),
    }
}

fn wanted(libraries: &[String]) -> [(&'static str, Value); 4] {
    [
        (SCHEMA_KEY, json!(SCHEMA)),
        (VERSION_KEY, json!(VERSION)),
        (PATH_KEY, json!(PATHS)),
        (LIBRARY_KEY, json!(libraries)),
    ]
}

fn merge(settings: &mut Map<String, Value>, key: &str, value: Value) {
    if let (Some(Value::Array(kept)), Value::Array(added)) = (settings.get_mut(key), &value) {
        for item in added {
            if !kept.contains(item) {
                kept.push(item.clone());
            }
        }
        return;
    }

    settings.insert(key.to_string(), value);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settings(text: &str) -> Map<String, Value> {
        parsed(text).unwrap()
    }

    #[test]
    fn existing_keys_stay_and_lists_grow() {
        let existing = r#"{
            "diagnostics.globals": ["vim"],
            "runtime.version": "LuaJIT",
            "workspace.library": ["/usr/share/lua", "meta"]
        }"#;

        let settings = settings(
            &merged(
                Some(existing),
                &[DEFINITIONS_DIR.to_string(), "~/plugins/lazyld".to_string()],
            )
            .unwrap(),
        );

        assert_eq!(settings["diagnostics.globals"], json!(["vim"]));
        assert_eq!(settings[VERSION_KEY], VERSION);
        assert_eq!(
            settings[LIBRARY_KEY],
            json!(["/usr/share/lua", "meta", "~/plugins/lazyld"])
        );
        assert_eq!(settings[PATH_KEY], json!(PATHS));
    }

    #[test]
    fn directories_follow_the_definitions() {
        let home = Path::new("/home/u");

        assert_eq!(
            libraries(
                home,
                Path::new("/home/u/.local/share/luadot"),
                &[
                    PathBuf::from("/home/u/.local/share/luadot/plugins/lazyld"),
                    PathBuf::from("/opt/lazyld"),
                ],
            ),
            [
                "~/.local/share/luadot/meta",
                "~/.local/share/luadot/plugins/lazyld",
                "/opt/lazyld",
            ]
        );
    }
}
