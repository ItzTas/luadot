use anyhow::Result;

use super::custom::Call;

#[derive(Debug, Clone)]
pub struct Task {
    about: Option<String>,
    run: Call,
}

impl Task {
    pub fn new(about: Option<String>, run: Call) -> Self {
        Self { about, run }
    }

    pub fn about(&self) -> Option<&str> {
        self.about.as_deref()
    }

    pub fn run(&self, what: &str, args: Vec<String>) -> Result<Option<String>> {
        self.run.run(what, args)
    }
}

#[cfg(test)]
mod tests {
    use mlua::Lua;

    use super::*;

    #[test]
    fn a_task_hands_its_arguments_to_the_function_as_a_list() {
        let lua = Lua::new();
        let function = lua
            .load(r#"return function(argv) return #argv .. ":" .. table.concat(argv, ",") end"#)
            .eval()
            .unwrap();
        let task = Task::new(None, Call::new(function));

        let shown = task
            .run("task `plug`", vec!["sync".to_string(), "--all".to_string()])
            .unwrap();

        assert_eq!(shown, Some("2:sync,--all".to_string()));
    }
}
