use mlua::{Function, Lua, Table};

use super::super::constants::API;
use super::super::parse::{conflict_policy, external, link_mode, matcher, mode_bits, owner_name};
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
    let pattern = matcher(entry)?;
    let link: Option<String> = entry.get("link")?;
    let conflict: Option<String> = entry.get("conflict")?;
    let on_change: Option<String> = entry.get("on_change")?;
    let ignore: Option<bool> = entry.get("ignore")?;
    let mode: Option<String> = entry.get("mode")?;
    let owner: Option<String> = entry.get("owner")?;
    let encrypt: Option<bool> = entry.get("encrypt")?;

    Ok(
        Rule::new(pattern, link_mode(link)?, conflict_policy(conflict)?)
            .with_on_change(on_change)
            .with_ignore(ignore)
            .with_mode(mode.map(|raw| mode_bits(&raw, "a rule")).transpose()?)
            .with_owner(owner.map(|raw| owner_name(&raw, "a rule")).transpose()?)
            .with_encrypt(encrypt),
    )
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
    fn a_rule_names_the_command_the_files_it_matches_run() {
        let config = configure(
            r#"
            ld.rules({
              { match = ".config/**", on_change = "notify-send updated" },
              { match = ".config/mako/**", on_change = "makoctl reload" },
            })
            "#,
        );

        assert_eq!(
            config.on_change(Path::new(".config/mako/config")),
            Some("makoctl reload")
        );
        assert_eq!(
            config.on_change(Path::new(".config/zsh/.zshrc")),
            Some("notify-send updated")
        );
        assert_eq!(config.on_change(Path::new(".bashrc")), None);
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
    fn both_forms_land_in_the_same_configuration() {
        let config = configure(
            r#"
            ld.rules({ match = ".ssh/**", conflict = "skip" })
            ld.rules({
              { match = ".config/nvim/**", link = "symbolic" },
            })
            "#,
        );

        assert_eq!(
            config.conflict_policy(Path::new(".ssh/config")),
            ConflictPolicy::Skip
        );
        assert_eq!(
            config.link_mode(Path::new(".config/nvim/init.lua")),
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
    fn ignoring_a_directory_ignores_its_contents() {
        let config = configure(r#"ld.rules({ match = ".config/nvim", ignore = true })"#);

        assert!(config.is_ignored(Path::new(".config/nvim")));
        assert!(config.is_ignored(Path::new(".config/nvim/lua/plugins.lua")));
        assert!(!config.is_ignored(Path::new(".config/zsh/.zshrc")));
    }

    #[test]
    fn a_single_star_does_not_cross_a_separator() {
        let config = configure(r#"ld.rules({ match = ".config/*", ignore = true })"#);

        assert!(config.is_ignored(Path::new(".config/mimeapps.list")));
        assert!(config.is_ignored(Path::new(".config/nvim/init.lua")));
        assert!(!config.is_ignored(Path::new(".bashrc")));
    }

    #[test]
    fn a_later_rule_takes_a_file_back_from_the_ignored_ones() {
        let config = configure(
            r#"
            ld.rules({
              { match = ".cache/**", ignore = true },
              { match = ".cache/keep/**", ignore = false },
            })
            "#,
        );

        assert!(config.is_ignored(Path::new(".cache/nvim/log")));
        assert!(!config.is_ignored(Path::new(".cache/keep/list")));
    }

    #[test]
    fn an_ignored_file_still_carries_the_other_keys_of_its_rule() {
        let config = configure(
            r#"
            ld.rules({ match = ".ssh/**", ignore = true, link = "symbolic" })
            "#,
        );
        let key = Path::new(".ssh/config");

        assert!(config.is_ignored(key));
        assert_eq!(config.link_mode(key), LinkMode::Symbolic);
    }

    #[test]
    fn a_rule_carries_a_mode_and_an_owner() {
        let config = configure(
            r#"
            ld.rules({
              { match = "root/etc/**", mode = "0644", owner = "root:root" },
              { match = "root/etc/sudoers.d/**", mode = "0440" },
            })
            "#,
        );

        let sudoers = Path::new("root/etc/sudoers.d/wheel");
        assert_eq!(config.mode(sudoers), Some(0o440));
        assert_eq!(config.owner(sudoers), Some("root:root"));

        let pacman = Path::new("root/etc/pacman.conf");
        assert_eq!(config.mode(pacman), Some(0o644));

        let bashrc = Path::new("home/.bashrc");
        assert_eq!(config.mode(bashrc), None);
        assert_eq!(config.owner(bashrc), None);
    }

    #[test]
    fn a_rule_marks_the_files_it_matches_as_encrypted() {
        let config = configure(
            r#"
            ld.rules({
              { match = "home/.ssh/id_*", encrypt = true },
              { match = "home/.config/*/secrets.toml", encrypt = true },
            })
            "#,
        );

        assert!(config.encrypt(Path::new("home/.ssh/id_ed25519")));
        assert!(config.encrypt(Path::new("home/.config/mail/secrets.toml")));
        assert!(!config.encrypt(Path::new("home/.ssh/config")));
        assert!(!config.encrypt(Path::new("home/.bashrc")));
    }

    #[test]
    fn a_later_rule_takes_a_file_back_from_the_encrypted_ones() {
        let config = configure(
            r#"
            ld.rules({
              { match = "home/.ssh/**", encrypt = true },
              { match = "home/.ssh/*.pub", encrypt = false },
            })
            "#,
        );

        assert!(config.encrypt(Path::new("home/.ssh/id_ed25519")));
        assert!(!config.encrypt(Path::new("home/.ssh/id_ed25519.pub")));
    }

    #[test]
    fn rejects_an_invalid_mode() {
        let err = format!(
            "{:#}",
            from_source(r#"ld.rules({ match = "root/etc/**", mode = "80" })"#).unwrap_err()
        );

        assert!(err.contains("three or four octal digits"));
    }

    #[test]
    fn rejects_an_invalid_owner() {
        let err = format!(
            "{:#}",
            from_source(r#"ld.rules({ match = "root/etc/**", owner = "a:b:c" })"#).unwrap_err()
        );

        assert!(err.contains("needs an `owner`"));
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

    #[test]
    fn a_regex_carries_every_key_a_glob_carries() {
        let config = configure(
            r#"
            ld.rules({
              { regex = "\\.sw[po]$", ignore = true },
              { regex = "^\\.config/mako/", on_change = "makoctl reload" },
            })
            "#,
        );

        assert!(config.is_ignored(Path::new(".vimrc.swp")));
        assert!(!config.is_ignored(Path::new(".vimrc")));
        assert_eq!(
            config.on_change(Path::new(".config/mako/config")),
            Some("makoctl reload")
        );
    }

    #[test]
    fn a_regex_rule_ranks_with_the_glob_ones() {
        let config = configure(
            r#"
            ld.rules({
              { match = ".config/**", link = "symbolic" },
              { regex = "^\\.config/nvim/", link = "hard" },
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
    fn a_regex_covers_the_subtree_of_a_directory_it_matches() {
        let config = configure(r#"ld.rules({ regex = "^\\.ssh$", link = "symbolic" })"#);

        assert_eq!(
            config.link_mode(Path::new(".ssh/keys/id_ed25519")),
            LinkMode::Symbolic
        );
    }

    #[test]
    fn a_match_takes_a_table_of_patterns() {
        let config = configure(
            r#"
            ld.rules({
              { match = { "**/*.tmp", "**/*.swp" }, ignore = true },
            })
            "#,
        );

        assert!(config.is_ignored(Path::new("home/.cache/build.tmp")));
        assert!(config.is_ignored(Path::new("home/.vimrc.swp")));
        assert!(!config.is_ignored(Path::new("home/.vimrc")));
    }

    #[test]
    fn a_regex_takes_a_table_of_expressions() {
        let config = configure(
            r#"
            ld.rules({
              { regex = { "^home/\\.local/state/", "\\.sw[po]$" }, ignore = true },
            })
            "#,
        );

        assert!(config.is_ignored(Path::new("home/.local/state/nvim/log")));
        assert!(config.is_ignored(Path::new("home/.vimrc.swp")));
        assert!(!config.is_ignored(Path::new("home/.local/share/list")));
    }

    #[test]
    fn a_table_of_patterns_carries_every_key_a_single_one_carries() {
        let config = configure(
            r#"
            ld.rules({
              { match = { "home/.ssh/**", "home/.gnupg/**" }, mode = "0600", encrypt = true },
            })
            "#,
        );

        for path in ["home/.ssh/id_ed25519", "home/.gnupg/private-keys"] {
            assert_eq!(config.mode(Path::new(path)), Some(0o600), "{path}");
            assert!(config.encrypt(Path::new(path)), "{path}");
        }
        assert_eq!(config.mode(Path::new("home/.bashrc")), None);
    }

    #[test]
    fn a_later_rule_takes_a_file_back_from_a_table_of_patterns() {
        let config = configure(
            r#"
            ld.rules({
              { match = { "home/.cache/**", "home/.config/**/Cache/**" }, ignore = true },
              { match = "home/.cache/keep/**", ignore = false },
            })
            "#,
        );

        assert!(config.is_ignored(Path::new("home/.cache/nvim/log")));
        assert!(config.is_ignored(Path::new("home/.config/chromium/Cache/data")));
        assert!(!config.is_ignored(Path::new("home/.cache/keep/list")));
    }

    #[test]
    fn rejects_an_empty_table_of_patterns() {
        let err = format!(
            "{:#}",
            from_source(r#"ld.rules({ match = {}, ignore = true })"#).unwrap_err()
        );

        assert!(err.contains("needs at least one pattern"));
    }

    #[test]
    fn rejects_a_pattern_that_is_neither_a_string_nor_a_table() {
        let err = format!(
            "{:#}",
            from_source(r#"ld.rules({ match = 7, ignore = true })"#).unwrap_err()
        );

        assert!(err.contains("takes a string or a table of strings"));
    }

    #[test]
    fn rejects_a_table_entry_that_is_not_a_string() {
        let err = format!(
            "{:#}",
            from_source(r#"ld.rules({ match = { ".ssh/**", true }, ignore = true })"#).unwrap_err()
        );

        assert!(err.contains("entry 2 is not a string"));
    }

    #[test]
    fn rejects_an_invalid_pattern_inside_a_table() {
        let err = format!(
            "{:#}",
            from_source(r#"ld.rules({ match = { ".ssh/**", "[" }, ignore = true })"#).unwrap_err()
        );

        assert!(err.contains("invalid pattern `[`"));
    }

    #[test]
    fn rejects_a_rule_carrying_both_syntaxes() {
        let err = format!(
            "{:#}",
            from_source(r#"ld.rules({ match = ".ssh/**", regex = "^\\.ssh/" })"#).unwrap_err()
        );

        assert!(err.contains("takes `match` or `regex`, not both"));
    }

    #[test]
    fn rejects_an_invalid_regex() {
        let err = format!(
            "{:#}",
            from_source(r#"ld.rules({ regex = "^[", ignore = true })"#).unwrap_err()
        );

        assert!(err.contains("invalid regex `^[`"));
    }

    #[test]
    fn rejects_an_invalid_pattern() {
        let err = format!(
            "{:#}",
            from_source(r#"ld.rules({ match = "[", ignore = true })"#).unwrap_err()
        );

        assert!(err.contains("invalid pattern `[`"));
    }

    #[test]
    fn rejects_a_rule_without_a_pattern() {
        let err = format!(
            "{:#}",
            from_source(r#"ld.rules({ { link = "hard" } })"#).unwrap_err()
        );

        assert!(err.contains("needs a `match` or `regex` pattern"));
    }
}
