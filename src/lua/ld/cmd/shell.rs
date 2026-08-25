use std::process::Command;

use mlua::{Function, Lua, Table};

use super::super::constants::API;
use super::super::exec::run;
use super::super::parse::external;
use super::constants::{NAMESPACE, SHELL, SHELL_ARG};

pub fn function(lua: &Lua) -> mlua::Result<Function> {
    lua.create_function(|_, (_, line): (Table, Option<String>)| {
        let line = line
            .ok_or_else(|| external(format!("`{API}.{NAMESPACE}` takes a command line to run")))?;

        let mut command = Command::new(SHELL);
        command.arg(SHELL_ARG).arg(&line);

        run(command, NAMESPACE, &line)
    })
}

#[cfg(test)]
mod tests {
    use super::super::fixture::eval;

    #[test]
    fn the_line_goes_through_the_shell() {
        assert_eq!(
            eval(r#"return cmd("printf 'a\nb\n' | tail -n 1")"#).unwrap(),
            "b"
        );
    }
}
