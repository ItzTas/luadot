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

    lua.create_function(move |_, args: Variadic<String>| {
        let mut command = Command::new(&name);
        command.args(args.iter());

        run(command, NAMESPACE, &display(&name, &args))
    })
}

#[cfg(test)]
mod tests {
    use super::super::fixture::eval;

    #[test]
    fn runs_the_program_with_arguments() {
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
}
