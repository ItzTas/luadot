use mlua::{Lua, Table, Value};

use super::super::constants::API;
use super::super::parse::{external, lookup};
use super::super::table::{Builder, build};
use super::constants::{
    BACKUP, BACKUP_DIR, BACKUP_KEEP, LINK, NAMESPACE, PKG_WARN, REPO_DIR, SETTERS,
};
use super::{backup, backup_dir, backup_keep, link, pkg_warn, repo_dir};

pub type Setter = fn(&Lua, Value) -> mlua::Result<()>;

pub fn table(lua: &Lua) -> mlua::Result<Table> {
    let functions: [(&str, Builder); 6] = [
        (BACKUP, backup::function),
        (BACKUP_DIR, backup_dir::function),
        (BACKUP_KEEP, backup_keep::function),
        (LINK, link::function),
        (PKG_WARN, pkg_warn::function),
        (REPO_DIR, repo_dir::function),
    ];

    let opt = build(lua, &functions)?;
    let meta = lua.create_table()?;
    meta.set(
        "__call",
        lua.create_function(|lua, (_, options): (Table, Table)| apply(lua, &options))?,
    )?;
    opt.set_metatable(Some(meta))?;

    Ok(opt)
}

fn apply(lua: &Lua, options: &Table) -> mlua::Result<()> {
    for pair in options.clone().pairs::<String, Value>() {
        let (name, value) =
            pair.map_err(|_| external(format!("`{API}.{NAMESPACE}` takes a table of options")))?;
        lookup(&SETTERS, &name, "option")?(lua, value)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::files::LinkMode;
    use crate::lua::from_source;

    #[test]
    fn a_table_call_sets_every_option_it_carries() {
        let config = from_source(
            r#"ld.opt({ link = "symbolic", pkg_warn = false, backup = false, backup_dir = "~/saved", backup_keep = 3 })"#,
        )
        .unwrap();

        assert_eq!(config.link_mode(Path::new(".bashrc")), LinkMode::Symbolic);
        assert!(!config.pkg_warn());
        assert!(!config.backup());
        assert_eq!(config.backup_dir(), Some(Path::new("~/saved")));
        assert_eq!(config.backup_keep(), Some(3));
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
        assert!(config.backup());
        assert_eq!(config.backup_dir(), None);
        assert_eq!(config.backup_keep(), None);
    }

    #[test]
    fn rejects_an_unknown_option() {
        let err = format!(
            "{:#}",
            from_source(r#"ld.opt({ lnik = "hard" })"#).unwrap_err()
        );

        assert!(err.contains("unknown option `lnik`"));
        assert!(
            err.contains("available: backup, backup_dir, backup_keep, link, pkg_warn, repo_dir")
        );
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
