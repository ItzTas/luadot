use mlua::{Function, Lua, Table, Value};

use super::super::constants::{API, CONFLICT, LINK, MATCH, MODE, ON_CHANGE};
use super::super::parse::{conflict_policy, external, link_mode, matcher, mode_bits, owner_name};
use super::constants::{AUTOCOMMIT, AUTOPUSH, ENCRYPT, IGNORE, LFS, OWNER, RULE_KEYS};
use crate::lua::{Config, Matcher, Rule};

pub fn function(lua: &Lua) -> mlua::Result<Function> {
    lua.create_function(|lua, list: Table| {
        let rules = rules(&list)?;
        Config::building(lua, |config| config.add_rules(rules))?;
        Ok(())
    })
}

fn rules(list: &Table) -> mlua::Result<Vec<Rule>> {
    if list.is_empty() {
        return Ok(Vec::new());
    }
    if list.raw_len() == 0 {
        let rule = rule(list).map_err(|err| external(format!("`{API}.rules`: {err}")))?;
        return Ok(vec![rule]);
    }

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
    known(entry)?;

    let pattern = matcher(entry)?;
    let link: Option<String> = entry.get(LINK)?;
    let conflict: Option<String> = entry.get(CONFLICT)?;
    let on_change: Option<String> = entry.get(ON_CHANGE)?;
    let ignore: Option<bool> = entry.get(IGNORE)?;
    let mode: Option<String> = entry.get(MODE)?;
    let owner: Option<String> = entry.get(OWNER)?;
    let encrypt: Option<bool> = entry.get(ENCRYPT)?;
    let lfs: Option<bool> = entry.get(LFS)?;
    let autocommit: Option<bool> = entry.get(AUTOCOMMIT)?;
    let autopush: Option<bool> = entry.get(AUTOPUSH)?;
    tracked(&pattern, lfs, encrypt)?;

    Ok(
        Rule::new(pattern, link_mode(link)?, conflict_policy(conflict)?)
            .with_on_change(on_change)
            .with_ignore(ignore)
            .with_mode(mode.map(|raw| mode_bits(&raw, "a rule")).transpose()?)
            .with_owner(owner.map(|raw| owner_name(&raw, "a rule")).transpose()?)
            .with_encrypt(encrypt)
            .with_lfs(lfs)
            .with_autocommit(autocommit)
            .with_autopush(autopush),
    )
}

fn tracked(pattern: &Matcher, lfs: Option<bool>, encrypt: Option<bool>) -> mlua::Result<()> {
    if lfs.is_none() {
        return Ok(());
    }
    if expressive(pattern) {
        return Err(external(format!(
            "`{LFS}` needs a `{MATCH}` pattern, git attributes have no regular expressions"
        )));
    }
    if encrypt == Some(true) && lfs == Some(true) {
        return Err(external(format!(
            "a rule takes `{ENCRYPT}` or `{LFS}`, not both"
        )));
    }

    Ok(())
}

fn expressive(pattern: &Matcher) -> bool {
    match pattern {
        Matcher::Regex(_) => true,
        Matcher::Any(matchers) => matchers.iter().any(expressive),
        Matcher::Glob(_) => false,
    }
}

fn known(entry: &Table) -> mlua::Result<()> {
    for pair in entry.clone().pairs::<String, Value>() {
        let (key, _) = pair.map_err(|_| external("a rule takes named keys"))?;

        if !RULE_KEYS.contains(&key.as_str()) {
            return Err(external(format!(
                "unknown key `{key}` (available: {})",
                RULE_KEYS.join(", ")
            )));
        }
    }

    Ok(())
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
    fn a_rule_marks_the_files_it_matches_as_never_managed() {
        let config = configure(
            r#"
            ld.rules({
              { match = "*.swp", ignore = true },
              { match = ".cache/**", ignore = true },
            })
            "#,
        );

        assert!(config.is_ignored(Path::new(".vimrc.swp")));
        assert!(config.is_ignored(Path::new(".cache/nvim/log")));
        assert!(!config.is_ignored(Path::new(".vimrc")));
    }

    #[test]
    fn a_rule_carries_a_mode_and_an_owner() {
        let config = configure(
            r#"
            ld.rules({
              { match = ".ssh/**", mode = "0644", owner = "me:wheel" },
              { match = ".ssh/id_*", mode = "0600" },
            })
            "#,
        );

        let key = Path::new(".ssh/id_ed25519");
        assert_eq!(config.mode(key), Some(0o600));
        assert_eq!(config.owner(key), Some("me:wheel"));

        let known = Path::new(".ssh/known_hosts");
        assert_eq!(config.mode(known), Some(0o644));

        let bashrc = Path::new(".bashrc");
        assert_eq!(config.mode(bashrc), None);
        assert_eq!(config.owner(bashrc), None);
    }

    #[test]
    fn a_rule_marks_the_files_it_matches_as_encrypted() {
        let config = configure(
            r#"
            ld.rules({
              { match = ".ssh/id_*", encrypt = true },
              { match = ".config/*/secrets.toml", encrypt = true },
            })
            "#,
        );

        assert!(config.encrypt(Path::new(".ssh/id_ed25519")));
        assert!(config.encrypt(Path::new(".config/mail/secrets.toml")));
        assert!(!config.encrypt(Path::new(".ssh/config")));
        assert!(!config.encrypt(Path::new(".bashrc")));
    }

    #[test]
    fn a_rule_sends_the_files_it_matches_to_lfs_in_the_order_it_was_declared() {
        let config = configure(
            r#"
            ld.rules({
              { match = { "Videos/**", "*.iso" }, lfs = true },
              { match = "Videos/notes/**", lfs = false },
            })
            "#,
        );

        assert_eq!(
            config.lfs_patterns(),
            [
                ("Videos/**".to_string(), true),
                ("*.iso".to_string(), true),
                ("Videos/notes/**".to_string(), false),
            ]
        );
    }

    #[test]
    fn rejects_a_rule_encrypting_what_it_sends_to_lfs() {
        let err = format!(
            "{:#}",
            from_source(r#"ld.rules({ match = ".secret.iso", lfs = true, encrypt = true })"#)
                .unwrap_err()
        );

        assert!(err.contains("takes `encrypt` or `lfs`, not both"));
    }

    #[test]
    fn a_regex_matches_what_a_glob_cannot_express() {
        let config = configure(
            r#"
            ld.rules({ regex = "^\\.config/(nvim|zsh)/", link = "symbolic" })
            "#,
        );

        assert_eq!(
            config.link_mode(Path::new(".config/nvim/init.lua")),
            LinkMode::Symbolic
        );
        assert_eq!(
            config.link_mode(Path::new(".config/zsh/.zshrc")),
            LinkMode::Symbolic
        );
        assert_eq!(
            config.link_mode(Path::new(".config/mako/config")),
            LinkMode::Hard
        );
    }
}
