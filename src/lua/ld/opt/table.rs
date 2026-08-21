use mlua::{Lua, Table};

use super::super::table::options;
use super::constants::{NAMESPACE, SETTERS};

pub fn table(lua: &Lua) -> mlua::Result<Table> {
    options(lua, NAMESPACE, &SETTERS, "option")
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::files::LinkMode;
    use crate::lua::from_source;

    #[test]
    fn a_table_call_sets_every_option_it_carries() {
        let config = from_source(
            r#"ld.opt({ link = "symbolic", pkg_warn = false, lfs = false, backup = false, backup_dir = "~/saved", backup_keep = 3, backup_age = "30d", autocommit = true, autopush = true })"#,
        )
        .unwrap();

        assert_eq!(config.link_mode(Path::new(".bashrc")), LinkMode::Symbolic);
        assert!(!config.pkg_warn());
        assert!(!config.lfs());
        assert!(config.autocommit(Path::new(".bashrc")));
        assert!(config.autopush(Path::new(".bashrc")));
        assert!(!config.backup());
        assert_eq!(config.backup_dir(), Some(Path::new("~/saved")));
        assert_eq!(config.backup_keep(), Some(3));
        assert_eq!(config.backup_age(), Some(2_592_000));
    }

    #[test]
    fn a_table_call_only_touches_the_options_it_carries() {
        let config = from_source(
            r#"
            ld.opt.link("symbolic")
            ld.opt({})
            "#,
        )
        .unwrap();

        assert_eq!(config.link_mode(Path::new(".bashrc")), LinkMode::Symbolic);
        assert!(config.pkg_warn());
        assert!(config.lfs());
        assert!(!config.autocommit(Path::new(".bashrc")));
        assert!(config.backup());
        assert_eq!(config.backup_dir(), None);
        assert_eq!(config.backup_keep(), None);
        assert_eq!(config.backup_age(), None);
    }

    #[test]
    fn rejects_an_unknown_option() {
        let err = format!(
            "{:#}",
            from_source(r#"ld.opt({ lnik = "hard" })"#).unwrap_err()
        );

        assert!(err.contains("unknown option `lnik`"));
        assert!(err.contains(
            "available: autocommit, autopush, backup, backup_age, backup_dir, backup_keep, conflict, lfs, link, passphrase_warn, pkg_warn, repo_dir"
        ));
    }

    #[test]
    fn rejects_a_value_the_option_does_not_accept() {
        let err = format!("{:#}", from_source("ld.opt({ link = {} })").unwrap_err());

        assert!(err.contains("`ld.opt.link` takes a string"));
    }

    #[test]
    fn reports_a_value_the_option_does_not_accept() {
        let err = format!(
            "{:#}",
            from_source(r#"ld.opt({ link = "magic" })"#).unwrap_err()
        );

        assert!(err.contains("unknown link mode `magic`"));
    }
}
