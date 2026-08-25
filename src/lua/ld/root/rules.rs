use mlua::{Function, Lua, Table, Value};

use super::super::constants::{API, CONFLICT, LINK, MATCH, MODE, ON_CHANGE};
use super::super::parse::{
    conflict_policy, external, link_mode, matcher, mode_bits, owner_name, track,
};
use super::constants::{AUTOCOMMIT, AUTOPUSH, ENCRYPT, LFS, OWNER, RULE_KEYS, TRACK, WHOLE};
use crate::files::LinkMode;
use crate::lua::{Config, Matcher, Rule, Track};

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
    let track = track(entry.get(TRACK)?)?;
    let mode: Option<String> = entry.get(MODE)?;
    let owner: Option<String> = entry.get(OWNER)?;
    let encrypt: Option<bool> = entry.get(ENCRYPT)?;
    let lfs: Option<bool> = entry.get(LFS)?;
    let autocommit: Option<bool> = entry.get(AUTOCOMMIT)?;
    let autopush: Option<bool> = entry.get(AUTOPUSH)?;
    let whole: Option<bool> = entry.get(WHOLE)?;
    tracked(&pattern, lfs, encrypt)?;
    placed_whole(whole, link.as_deref(), encrypt)?;
    adopted(&pattern, track)?;

    Ok(
        Rule::new(pattern, link_mode(link)?, conflict_policy(conflict)?)
            .with_on_change(on_change)
            .with_track(track)
            .with_mode(mode.map(|raw| mode_bits(&raw, "a rule")).transpose()?)
            .with_owner(owner.map(|raw| owner_name(&raw, "a rule")).transpose()?)
            .with_encrypt(encrypt)
            .with_lfs(lfs)
            .with_autocommit(autocommit)
            .with_autopush(autopush)
            .with_whole(whole),
    )
}

fn placed_whole(
    whole: Option<bool>,
    link: Option<&str>,
    encrypt: Option<bool>,
) -> mlua::Result<()> {
    if whole != Some(true) {
        return Ok(());
    }
    if link == Some(LinkMode::Hard.name()) {
        return Err(external(format!(
            "a rule placing directories `{WHOLE}` takes `{LINK}` \"{}\" or \"{}\"",
            LinkMode::Symbolic.name(),
            LinkMode::Copy.name()
        )));
    }
    if encrypt == Some(true) {
        return Err(external(format!(
            "a rule takes `{WHOLE}` or `{ENCRYPT}`, not both"
        )));
    }

    Ok(())
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

fn adopted(pattern: &Matcher, track: Option<Track>) -> mlua::Result<()> {
    if track != Some(Track::Auto) || pattern.rooted() {
        return Ok(());
    }

    Err(external(format!(
        "`{TRACK}` = `{}` needs a `{MATCH}` pattern opening on a name, luadot looks under it instead of walking your whole home",
        Track::Auto.name()
    )))
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
    use crate::lua::{Config, Track, from_source};

    fn configure(source: &str) -> Config {
        from_source(source).unwrap()
    }

    #[test]
    fn overrides_the_defaults() {
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
    fn a_rule_marks_files_unmanaged() {
        let config = configure(
            r#"
            ld.rules({
              { match = "*.swp", track = "never" },
              { match = ".cache/**", track = "never" },
            })
            "#,
        );

        assert!(config.is_ignored(Path::new(".vimrc.swp")));
        assert!(config.is_ignored(Path::new(".cache/nvim/log")));
        assert!(!config.is_ignored(Path::new(".vimrc")));
    }

    #[test]
    fn a_rule_marks_files_adopted() {
        let config = configure(
            r#"
            ld.rules({
              { match = ".config/nvim/**", track = "auto" },
              { match = ".config/nvim/spell/**", track = "manual" },
            })
            "#,
        );

        assert_eq!(
            config.track(Path::new(".config/nvim/init.lua")),
            Track::Auto
        );
        assert_eq!(
            config.track(Path::new(".config/nvim/spell/en.add")),
            Track::Manual
        );
        assert_eq!(config.track(Path::new(".bashrc")), Track::Manual);
        assert_eq!(config.adoption_roots(), [PathBuf::from(".config/nvim")]);
    }

    #[test]
    fn rejects_an_adopted_pattern_without_a_root() {
        for pattern in [r#"match = "**/*.toml""#, r#"regex = "\\.toml$""#] {
            let err = format!(
                "{:#}",
                from_source(&format!(r#"ld.rules({{ {pattern}, track = "auto" }})"#)).unwrap_err()
            );

            assert!(err.contains("opening on a name"), "{pattern}");
        }
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
    fn a_rule_marks_files_encrypted() {
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
    fn a_rule_sends_files_to_lfs() {
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
    fn rejects_a_whole_rule_hard_linking() {
        let err = format!(
            "{:#}",
            from_source(r#"ld.rules({ match = ".config/nvim", whole = true, link = "hard" })"#)
                .unwrap_err()
        );

        assert!(err.contains("takes `link` \"symbolic\" or \"copy\""));
    }

    #[test]
    fn rejects_a_whole_rule_encrypting() {
        let err = format!(
            "{:#}",
            from_source(r#"ld.rules({ match = ".gnupg", whole = true, encrypt = true })"#)
                .unwrap_err()
        );

        assert!(err.contains("takes `whole` or `encrypt`, not both"));
    }

    #[test]
    fn rejects_encrypt_with_lfs() {
        let err = format!(
            "{:#}",
            from_source(r#"ld.rules({ match = ".secret.iso", lfs = true, encrypt = true })"#)
                .unwrap_err()
        );

        assert!(err.contains("takes `encrypt` or `lfs`, not both"));
    }

    #[test]
    fn a_regex_matches_beyond_a_glob() {
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
