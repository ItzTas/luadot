use std::path::Path;

use serde::de::Error;
use serde_json::{Map, Value, json};

use super::constants::{
    DEFINITIONS_DIR, LIBRARY_KEY, NOT_AN_OBJECT, PATH_KEY, PATHS, SCHEMA, SCHEMA_KEY, TILDE,
    VERSION, VERSION_KEY,
};

pub fn merged(existing: Option<&str>, library: &str) -> serde_json::Result<String> {
    let mut settings = match existing {
        Some(text) => parsed(text)?,
        None => Map::new(),
    };
    for (key, value) in wanted(library) {
        merge(&mut settings, key, value);
    }

    let mut text = serde_json::to_string_pretty(&Value::Object(settings))?;
    text.push('\n');

    Ok(text)
}

pub fn library(home: &Path, data: &Path) -> String {
    let definitions = data.join(DEFINITIONS_DIR);

    match definitions.strip_prefix(home) {
        Ok(relative) => Path::new(TILDE).join(relative).display().to_string(),
        Err(_) => definitions.display().to_string(),
    }
}

fn parsed(text: &str) -> serde_json::Result<Map<String, Value>> {
    match serde_json::from_str(text)? {
        Value::Object(settings) => Ok(settings),
        _ => Err(serde_json::Error::custom(NOT_AN_OBJECT)),
    }
}

fn wanted(library: &str) -> [(&'static str, Value); 4] {
    [
        (SCHEMA_KEY, json!(SCHEMA)),
        (VERSION_KEY, json!(VERSION)),
        (PATH_KEY, json!(PATHS)),
        (LIBRARY_KEY, json!([library])),
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
    fn a_missing_file_gets_every_setting() {
        let settings = settings(&merged(None, DEFINITIONS_DIR).unwrap());

        assert_eq!(settings[SCHEMA_KEY], SCHEMA);
        assert_eq!(settings[VERSION_KEY], VERSION);
        assert_eq!(settings[PATH_KEY], json!(PATHS));
        assert_eq!(settings[LIBRARY_KEY], json!([DEFINITIONS_DIR]));
    }

    #[test]
    fn an_existing_file_keeps_its_keys_and_its_lists_grow() {
        let existing = r#"{
            "diagnostics.globals": ["vim"],
            "runtime.version": "LuaJIT",
            "workspace.library": ["/usr/share/lua", "meta"]
        }"#;

        let settings = settings(&merged(Some(existing), DEFINITIONS_DIR).unwrap());

        assert_eq!(settings["diagnostics.globals"], json!(["vim"]));
        assert_eq!(settings[VERSION_KEY], VERSION);
        assert_eq!(settings[LIBRARY_KEY], json!(["/usr/share/lua", "meta"]));
        assert_eq!(settings[PATH_KEY], json!(PATHS));
    }

    #[test]
    fn a_file_that_is_not_a_json_object_is_refused() {
        assert!(merged(Some("{ // a comment\n}"), DEFINITIONS_DIR).is_err());
        assert!(merged(Some("[]"), DEFINITIONS_DIR).is_err());
    }

    #[test]
    fn the_text_ends_with_a_newline() {
        assert!(merged(None, DEFINITIONS_DIR).unwrap().ends_with("}\n"));
    }

    #[test]
    fn the_shared_definitions_are_named_through_the_home_directory() {
        let home = Path::new("/home/u");

        assert_eq!(
            library(home, Path::new("/home/u/.local/share/luadot")),
            "~/.local/share/luadot/meta"
        );
        assert_eq!(
            library(home, Path::new("/usr/share/luadot")),
            "/usr/share/luadot/meta"
        );
    }
}
