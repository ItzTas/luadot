use mlua::{Function, Lua, Value, Variadic};

use super::super::constants::API;
use super::super::value::path;
use super::constants::{AT, NAMESPACE};
use super::run::run_in;
use crate::lua::Scope;
use crate::utils::expand;

pub fn function(lua: &Lua) -> mlua::Result<Function> {
    lua.create_function(|lua, value: Value| {
        let raw = path(NAMESPACE, &value, AT, "a directory")?;
        let home = Scope::building(lua)?.home().to_path_buf();
        let dir = expand(&home, &raw);
        let command = format!("`{API}.{NAMESPACE}.{AT}`");

        lua.create_function(move |lua, args: Variadic<String>| run_in(lua, &dir, &args, &command))
    })
}

#[cfg(test)]
mod tests {
    use super::super::fixture::eval;
    use crate::git::fixture::repository;

    #[test]
    fn runs_git_inside_the_directory_it_was_given() {
        let repo = repository();
        std::fs::write(repo.path().join("tracked"), "one\n").unwrap();

        let output = eval(
            repo.path(),
            r#"return git.at("~")("status", "--porcelain")"#,
        )
        .unwrap();

        assert_eq!(output, "?? tracked");
    }

    #[test]
    fn reports_a_call_without_arguments() {
        let repo = repository();

        let err = eval(repo.path(), r#"return git.at("~")()"#)
            .unwrap_err()
            .to_string();

        assert!(err.contains("`ld.git.at` takes the arguments of the git command to run"));
    }
}
