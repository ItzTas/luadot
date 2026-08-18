use mlua::{Lua, Value};

use super::super::constants::API;
use super::super::parse::external;
use super::super::surface::{self, Surface};
use super::super::value::span;
use super::constants::{BACKUP, BACKUP_AGE, NAMESPACE, SPAN_KIND};
use crate::lua::Config;

pub fn set(lua: &Lua, value: Value) -> mlua::Result<()> {
    if surface::inert(lua, &format!("{NAMESPACE}.{BACKUP_AGE}"), Surface::Config) {
        return Ok(());
    }

    let age = span(NAMESPACE, &value, BACKUP_AGE, SPAN_KIND)?;
    if age == 0 {
        return Err(external(format!(
            "`{API}.{NAMESPACE}.{BACKUP_AGE}` takes one second or more; `{API}.{NAMESPACE}.{BACKUP}(false)` is how backups are turned off"
        )));
    }

    Config::building(lua)?.set_backup_age(age);
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua::from_source;

    #[test]
    fn every_backup_is_kept_until_an_age_is_set() {
        let config = from_source("local unused = 1").unwrap();

        assert_eq!(config.backup_age(), None);
    }

    #[test]
    fn takes_the_age_a_backup_is_kept_for() {
        let config = from_source(r#"ld.opt.backup_age("30d")"#).unwrap();

        assert_eq!(config.backup_age(), Some(2_592_000));
    }

    #[test]
    fn reads_every_unit_it_takes() {
        for (source, seconds) in [
            (r#"ld.opt.backup_age("45s")"#, 45),
            (r#"ld.opt.backup_age("90m")"#, 5_400),
            (r#"ld.opt.backup_age("12h")"#, 43_200),
            (r#"ld.opt.backup_age("2w")"#, 1_209_600),
        ] {
            let config = from_source(source).unwrap();

            assert_eq!(config.backup_age(), Some(seconds), "{source}");
        }
    }

    #[test]
    fn the_last_age_wins() {
        let config = from_source(
            r#"
            ld.opt.backup_age("1d")
            ld.opt.backup_age("7d")
            "#,
        )
        .unwrap();

        assert_eq!(config.backup_age(), Some(604_800));
    }

    #[test]
    fn rejects_keeping_nothing() {
        let err = format!(
            "{:#}",
            from_source(r#"ld.opt.backup_age("0d")"#).unwrap_err()
        );

        assert!(err.contains("`ld.opt.backup_age` takes one second or more"));
        assert!(err.contains("`ld.opt.backup(false)`"));
    }

    #[test]
    fn rejects_a_span_without_a_unit() {
        let err = format!(
            "{:#}",
            from_source(r#"ld.opt.backup_age("30")"#).unwrap_err()
        );

        assert!(err.contains("`ld.opt.backup_age` takes a span like \"30d\""));
        assert!(err.contains("got `30`"));
    }

    #[test]
    fn rejects_a_value_that_is_not_a_string() {
        let err = format!("{:#}", from_source("ld.opt.backup_age(30)").unwrap_err());

        assert!(err.contains("`ld.opt.backup_age` takes a string"));
    }
}
