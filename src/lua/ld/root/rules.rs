use mlua::{Function, Lua, Table};

use super::super::constants::{API, CONFLICT_POLICIES, LINK_MODES};
use super::super::parse::{external, lookup, pattern};
use super::super::surface::{self, Surface};
use super::constants::RULES;
use crate::lua::{Config, Rule};

pub fn function(lua: &Lua) -> mlua::Result<Function> {
    lua.create_function(|lua, list: Table| {
        if surface::inert(lua, RULES, Surface::Config) {
            return Ok(());
        }

        let rules = rules(&list)?;
        Config::building(lua)?.add_rules(rules);
        Ok(())
    })
}

fn rules(list: &Table) -> mlua::Result<Vec<Rule>> {
    list.clone()
        .sequence_values::<Table>()
        .enumerate()
        .map(|(index, entry)| {
            let entry = entry.map_err(|_| {
                external(format!("`{API}.rules` entry {} is not a table", index + 1))
            })?;
            rule(&entry)
                .map_err(|err| external(format!("`{API}.rules` entry {}: {err}", index + 1)))
        })
        .collect()
}

fn rule(entry: &Table) -> mlua::Result<Rule> {
    let raw: Option<String> = entry.get("match")?;
    let raw = raw.ok_or_else(|| external("a rule needs a `match` pattern"))?;
    let link: Option<String> = entry.get("link")?;
    let conflict: Option<String> = entry.get("conflict")?;

    Ok(Rule::new(
        pattern(&raw)?,
        link.map(|name| lookup(&LINK_MODES, &name, "link mode"))
            .transpose()?,
        conflict
            .map(|name| lookup(&CONFLICT_POLICIES, &name, "conflict policy"))
            .transpose()?,
    ))
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use crate::files::{ConflictPolicy, LinkMode};
    use crate::lua::{Config, from_source};

    fn configure(source: &str) -> Config {
        from_source(source).unwrap()
    }

    #[test]
    fn override_the_defaults_for_matching_files() {
        let config = configure(
            r#"
            ld.opt.link("hard")
            ld.rules({
              { match = ".config/nvim/**", link = "symbolic", conflict = "error" },
            })
            "#,
        );

        let nvim = PathBuf::from(".config/nvim/lua/plugins.lua");
        assert_eq!(config.link_mode(&nvim), LinkMode::Symbolic);
        assert_eq!(config.conflict_policy(&nvim), ConflictPolicy::Error);

        let other = Path::new(".bashrc");
        assert_eq!(config.link_mode(other), LinkMode::Hard);
        assert_eq!(config.conflict_policy(other), ConflictPolicy::Overwrite);
    }

    #[test]
    fn the_last_matching_rule_wins() {
        let config = configure(
            r#"
            ld.rules({
              { match = ".config/**", link = "symbolic" },
              { match = ".config/nvim/**", link = "hard" },
            })
            "#,
        );

        assert_eq!(
            config.link_mode(Path::new(".config/nvim/init.lua")),
            LinkMode::Hard
        );
        assert_eq!(
            config.link_mode(Path::new(".config/zsh/.zshrc")),
            LinkMode::Symbolic
        );
    }

    #[test]
    fn repeated_calls_accumulate() {
        let config = configure(
            r#"
            ld.rules({ { match = ".ssh/**", link = "symbolic" } })
            ld.rules({ { match = ".ssh/**", conflict = "skip" } })
            "#,
        );
        let key = Path::new(".ssh/config");

        assert_eq!(config.link_mode(key), LinkMode::Symbolic);
        assert_eq!(config.conflict_policy(key), ConflictPolicy::Skip);
    }

    #[test]
    fn a_rule_only_overrides_the_fields_it_sets() {
        let config = configure(
            r#"
            ld.git.conflict("skip")
            ld.rules({ { match = ".ssh/**", link = "symbolic" } })
            "#,
        );
        let key = Path::new(".ssh/config");

        assert_eq!(config.link_mode(key), LinkMode::Symbolic);
        assert_eq!(config.conflict_policy(key), ConflictPolicy::Skip);
    }

    #[test]
    fn a_rule_matching_a_directory_covers_its_subtree() {
        let config = configure(r#"ld.rules({ { match = ".ssh", link = "symbolic" } })"#);

        assert_eq!(
            config.link_mode(Path::new(".ssh/keys/id_ed25519")),
            LinkMode::Symbolic
        );
    }

    #[test]
    fn rejects_a_rule_without_a_pattern() {
        let err = format!(
            "{:#}",
            from_source(r#"ld.rules({ { link = "hard" } })"#).unwrap_err()
        );

        assert!(err.contains("needs a `match` pattern"));
    }
}
