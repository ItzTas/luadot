use std::process::Command;

use mlua::{Function, Lua, Table, Variadic};

use super::super::constants::API;
use super::super::exec::{display, run};
use super::super::parse::external;
use super::constants::NAMESPACE;

pub fn function(lua: &Lua) -> mlua::Result<Function> {
    lua.create_function(|lua, (_, name): (Table, String)| program(lua, name))
}

fn program(lua: &Lua, name: String) -> mlua::Result<Function> {
    if name.is_empty() {
        return Err(external(format!(
            "`{API}.{NAMESPACE}` was indexed with an empty program name"
        )));
    }

    lua.create_function(move |lua, args: Variadic<String>| {
        let mut command = Command::new(&name);
        command.args(args.iter());

        run(lua, command, NAMESPACE, &display(&name, &args))
    })
}

#[cfg(test)]
mod tests {
    use super::super::fixture::eval;

    #[test]
    fn runs_the_indexed_program_with_its_arguments() {
        assert_eq!(
            eval(r#"return cmd.printf("%s-%s", "one", "two")"#).unwrap(),
            "one-two"
        );
    }

    #[test]
    fn every_argument_stays_literal() {
        assert_eq!(
            eval(r#"return cmd.printf("[%s]", "one  two")"#).unwrap(),
            "[one  two]"
        );
    }

    #[test]
    fn a_failing_program_stops_the_script() {
        let err = eval(r#"return cmd["false"]()"#).unwrap_err().to_string();

        assert!(err.contains("`false` exited with status 1"));
    }

    #[test]
    fn reports_a_program_that_does_not_exist() {
        let err = eval(r#"return cmd["luadot-no-such-program"]()"#)
            .unwrap_err()
            .to_string();

        assert!(err.contains("failed to run `luadot-no-such-program`"));
    }

    #[test]
    fn reports_an_empty_program_name() {
        let err = eval(r#"return cmd[""]()"#).unwrap_err().to_string();

        assert!(err.contains("was indexed with an empty program name"));
    }
}
