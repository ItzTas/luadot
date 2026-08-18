use std::path::{Path, PathBuf};

use mlua::{IntoLua, Lua, Value};

use super::constants::{DEFAULT, PATH, SIDE, STATE, SYSTEM, TEMPLATES, TOTAL};
use crate::files::{FileStatus, Side};

#[derive(Debug, Clone)]
pub struct StatusFile {
    path: PathBuf,
    system: PathBuf,
    side: Side,
    state: FileStatus,
}

#[derive(Debug, Clone)]
pub struct StatusCounts {
    side: Side,
    total: usize,
    templates: usize,
    states: Vec<(FileStatus, u32)>,
    default: String,
}

impl StatusFile {
    pub fn new(path: PathBuf, system: PathBuf, side: Side, state: FileStatus) -> Self {
        Self {
            path,
            system,
            side,
            state,
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn state(&self) -> FileStatus {
        self.state
    }
}

impl StatusCounts {
    pub fn new(side: Side, total: usize, default: String) -> Self {
        Self {
            side,
            total,
            templates: 0,
            states: Vec::new(),
            default,
        }
    }

    pub fn with_templates(mut self, templates: usize) -> Self {
        self.templates = templates;
        self
    }

    pub fn with_states(mut self, states: Vec<(FileStatus, u32)>) -> Self {
        self.states = states;
        self
    }
}

impl IntoLua for &StatusFile {
    fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
        let file = lua.create_table()?;
        file.set(PATH, self.path.to_string_lossy().as_ref())?;
        file.set(SYSTEM, self.system.to_string_lossy().as_ref())?;
        file.set(SIDE, self.side.dir())?;
        file.set(STATE, self.state.name())?;

        Ok(Value::Table(file))
    }
}

impl IntoLua for &StatusCounts {
    fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
        let counts = lua.create_table()?;
        counts.set(SIDE, self.side.dir())?;
        counts.set(TOTAL, self.total)?;
        counts.set(TEMPLATES, self.templates)?;
        counts.set(DEFAULT, self.default.as_str())?;

        for (state, count) in &self.states {
            counts.set(state.name(), *count)?;
        }

        Ok(Value::Table(counts))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read(lua: &Lua, source: &str, value: impl IntoLua) -> String {
        lua.globals().set("subject", value).unwrap();

        lua.load(source).eval().unwrap()
    }

    #[test]
    fn a_file_carries_where_it_is_and_what_state_it_is_in() {
        let lua = Lua::new();
        let file = StatusFile::new(
            PathBuf::from("home/.bashrc"),
            PathBuf::from("/home/u/.bashrc"),
            Side::Repository,
            FileStatus::Unlinked,
        );

        assert_eq!(read(&lua, "return subject.path", &file), "home/.bashrc");
        assert_eq!(
            read(&lua, "return subject.system", &file),
            "/home/u/.bashrc"
        );
        assert_eq!(read(&lua, "return subject.side", &file), "repository");
        assert_eq!(read(&lua, "return subject.state", &file), "unlinked");
    }

    #[test]
    fn the_counts_carry_a_number_per_state_and_the_line_they_replace() {
        let lua = Lua::new();
        let counts = StatusCounts::new(
            Side::Repository,
            14,
            "14 managed file(s) (12 synced, 1 missing, 0 unlinked, 1 differs)".to_string(),
        )
        .with_states(vec![
            (FileStatus::Synced, 12),
            (FileStatus::Missing, 1),
            (FileStatus::Unlinked, 0),
            (FileStatus::Differs, 1),
            (FileStatus::Unreadable, 0),
        ]);

        assert_eq!(read(&lua, "return subject.side", &counts), "repository");
        assert_eq!(read(&lua, "return subject.total .. \"\"", &counts), "14");
        assert_eq!(read(&lua, "return subject.templates .. \"\"", &counts), "0");
        assert_eq!(read(&lua, "return subject.synced .. \"\"", &counts), "12");
        assert_eq!(read(&lua, "return subject.missing .. \"\"", &counts), "1");
        assert_eq!(
            read(&lua, "return subject.unreadable .. \"\"", &counts),
            "0"
        );
        assert_eq!(
            read(&lua, "return subject.default", &counts),
            "14 managed file(s) (12 synced, 1 missing, 0 unlinked, 1 differs)"
        );
    }

    #[test]
    fn the_generated_side_counts_the_templates_it_resolved() {
        let lua = Lua::new();
        let counts = StatusCounts::new(Side::Generated, 3, "unused".to_string()).with_templates(2);

        assert_eq!(read(&lua, "return subject.side", &counts), "generated");
        assert_eq!(read(&lua, "return subject.templates .. \"\"", &counts), "2");
        assert_eq!(read(&lua, "return subject.total .. \"\"", &counts), "3");
    }
}
