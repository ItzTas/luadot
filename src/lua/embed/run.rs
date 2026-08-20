use std::sync::{Arc, Mutex, MutexGuard};

use mlua::{Function, Lua, Table, Value};

use super::compile::Chunk;
use super::constants::{EMIT, WRITE};

pub fn run(lua: &Lua, chunk: Chunk, name: &str, environment: Table) -> mlua::Result<String> {
    let (source, literals) = chunk.into_parts();
    let buffer = Arc::new(Mutex::new(String::new()));

    environment.set(EMIT, emit(lua, literals, buffer.clone())?)?;
    environment.set(WRITE, write(lua, buffer.clone())?)?;

    lua.load(source)
        .set_name(name)
        .set_environment(environment)
        .exec()?;

    let rendered = lock(&buffer)?.clone();
    Ok(rendered)
}

fn emit(lua: &Lua, literals: Vec<String>, buffer: Arc<Mutex<String>>) -> mlua::Result<Function> {
    lua.create_function(move |_, index: usize| {
        let text = index
            .checked_sub(1)
            .and_then(|index| literals.get(index))
            .ok_or_else(|| mlua::Error::external(format!("no literal {index}")))?;
        lock(&buffer)?.push_str(text);
        Ok(())
    })
}

fn write(lua: &Lua, buffer: Arc<Mutex<String>>) -> mlua::Result<Function> {
    lua.create_function(move |lua, value: Value| {
        if value.is_nil() {
            return Err(refused_nil(lua));
        }
        let text = value.to_string()?;
        lock(&buffer)?.push_str(&text);
        Ok(())
    })
}

fn refused_nil(lua: &Lua) -> mlua::Error {
    let line = lua.inspect_stack(1, |debug| debug.current_line()).flatten();
    let place = line.map_or_else(String::new, |line| format!(" on line {line}"));
    mlua::Error::external(format!("the expression{place} was nil"))
}

fn lock(buffer: &Mutex<String>) -> mlua::Result<MutexGuard<'_, String>> {
    buffer
        .lock()
        .map_err(|_| mlua::Error::external("the render buffer was poisoned"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lua::embed::compile::compile;
    use crate::lua::runtime::runtime;

    fn render(source: &str) -> mlua::Result<String> {
        let lua = runtime().unwrap();
        let chunk = compile(source).unwrap();
        let environment = lua.create_table().unwrap();
        let meta = lua.create_table().unwrap();
        meta.set("__index", lua.globals()).unwrap();
        environment.set_metatable(Some(meta)).unwrap();

        run(&lua, chunk, "test", environment)
    }

    #[test]
    fn literals_and_expressions_come_out_in_order() {
        assert_eq!(
            render("export EDITOR=<%= \"nvim\" %>\n").unwrap(),
            "export EDITOR=nvim\n"
        );
    }

    #[test]
    fn statements_drive_what_is_emitted() {
        assert_eq!(
            render("<% for i = 1, 2 do -%>\nx=<%= i %>\n<% end -%>\n").unwrap(),
            "x=1\nx=2\n"
        );
    }

    #[test]
    fn a_return_mid_template_ends_it_early() {
        assert_eq!(render("a<% do return end %>b").unwrap(), "a");
    }

    #[test]
    fn a_runtime_error_reports_the_template_line() {
        let err = render("line\nline\n<% error(\"boom\") %>")
            .unwrap_err()
            .to_string();

        assert!(err.contains("boom"));
        assert!(err.contains(":3:"));
    }

    #[test]
    fn a_nil_expression_is_refused() {
        let err = render("line\n<%= nil %>").unwrap_err().to_string();

        assert!(err.contains("was nil"));
        assert!(err.contains("line 2"));
    }

    #[test]
    fn an_undefined_name_is_refused() {
        let err = render("export EDITOR=<%= edtior %>")
            .unwrap_err()
            .to_string();

        assert!(err.contains("was nil"));
    }

    #[test]
    fn a_false_expression_is_written() {
        assert_eq!(render("<%= false %>").unwrap(), "false");
    }
}
