use std::process::Command;

use mlua::{Function, Lua, Table};

use super::super::constants::API;
use super::super::exec::run;
use super::super::parse::external;
use super::constants::{NAMESPACE, SHELL, SHELL_ARG};

pub fn function(lua: &Lua) -> mlua::Result<Function> {
    lua.create_function(|lua, (_, line): (Table, Option<String>)| {
        let line = line
            .ok_or_else(|| external(format!("`{API}.{NAMESPACE}` takes a command line to run")))?;

        let mut command = Command::new(SHELL);
        command.arg(SHELL_ARG).arg(&line);

        run(lua, command, NAMESPACE, &line)
    })
}

#[cfg(test)]
mod tests {
    use super::super::table::table;
    use crate::lua::runtime::runtime;

    fn eval(source: &str) -> mlua::Result<String> {
        let lua = runtime().unwrap();
        let cmd = table(&lua).unwrap();
        lua.globals().set("cmd", cmd).unwrap();

        lua.load(source).eval()
    }

    #[test]
    fn captures_what_the_command_prints() {
        assert_eq!(eval(r#"return cmd("printf hello")"#).unwrap(), "hello");
    }

    #[test]
    fn the_line_goes_through_the_shell() {
        assert_eq!(
            eval(r#"return cmd("printf 'a\nb\n' | tail -n 1")"#).unwrap(),
            "b"
        );
    }

    #[test]
    fn a_failing_command_stops_the_script() {
        let err = eval(r#"return cmd("exit 4")"#).unwrap_err().to_string();

        assert!(err.contains("`exit 4` exited with status 4"));
    }

    #[test]
    fn reports_a_call_without_a_command() {
        let err = eval("return cmd()").unwrap_err().to_string();

        assert!(err.contains("`ld.cmd` takes a command line to run"));
    }
}
