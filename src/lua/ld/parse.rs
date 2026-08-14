use glob::Pattern;

pub fn external(message: impl Into<String>) -> mlua::Error {
    mlua::Error::external(message.into())
}

pub fn chain(err: anyhow::Error) -> mlua::Error {
    external(format!("{err:#}"))
}

pub fn pattern(raw: &str) -> mlua::Result<Pattern> {
    Pattern::new(raw).map_err(|err| external(format!("invalid pattern `{raw}`: {err}")))
}

pub fn lookup<T: Copy>(entries: &[(&str, T)], name: &str, field: &str) -> mlua::Result<T> {
    entries
        .iter()
        .find(|(key, _)| *key == name)
        .map(|(_, value)| *value)
        .ok_or_else(|| {
            external(format!(
                "unknown {field} `{name}` (available: {})",
                keys(entries)
            ))
        })
}

fn keys<T>(entries: &[(&str, T)]) -> String {
    entries
        .iter()
        .map(|(key, _)| *key)
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;

    use super::super::constants::{CONFLICT_POLICIES, LINK_MODES};

    const ENTRIES: [(&str, u8); 2] = [("one", 1), ("two", 2)];

    #[test]
    fn link_modes_round_trip_through_their_name() {
        for (name, mode) in LINK_MODES {
            assert_eq!(mode.name(), name);
        }
    }

    #[test]
    fn conflict_policies_round_trip_through_their_name() {
        for (name, policy) in CONFLICT_POLICIES {
            assert_eq!(policy.name(), name);
        }
    }

    #[test]
    fn lookup_finds_a_known_name() {
        assert_eq!(lookup(&ENTRIES, "two", "number").unwrap(), 2);
    }

    #[test]
    fn lookup_lists_the_available_names() {
        let err = lookup(&ENTRIES, "three", "number").unwrap_err().to_string();

        assert!(err.contains("unknown number `three`"));
        assert!(err.contains("available: one, two"));
    }

    #[test]
    fn pattern_reports_an_invalid_glob() {
        assert!(
            pattern("[")
                .unwrap_err()
                .to_string()
                .contains("invalid pattern `[`")
        );
    }
}
