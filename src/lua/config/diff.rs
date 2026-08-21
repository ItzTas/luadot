use std::path::PathBuf;

use mlua::{IntoLua, Lua, Value};

use super::constants::{CONTENT, DEFAULT, DIFF_STATES, DRIFTED, MODE, SIDE, SOURCE, SYSTEM, TOTAL};
use super::custom::Custom;
use super::file;
use super::report::Report;
use crate::files::Side;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffState {
    Missing,
    Differs,
    Mode,
    Other,
}

#[derive(Debug, Clone)]
pub struct Tool {
    program: String,
    arguments: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct Diff {
    report: Report,
    tool: Option<Tool>,
    args: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct DiffFile {
    path: PathBuf,
    system: PathBuf,
    side: Side,
    state: DiffState,
    content: Vec<u8>,
    mode: u32,
    found: Option<Vec<u8>>,
    found_mode: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct DiffCounts {
    side: Side,
    drifted: usize,
    total: usize,
    default: String,
}

impl DiffState {
    pub fn name(self) -> &'static str {
        DIFF_STATES
            .iter()
            .find(|(_, state)| *state == self)
            .map(|(name, _)| *name)
            .unwrap_or_default()
    }

    pub fn staged(self) -> bool {
        matches!(self, Self::Missing | Self::Differs)
    }
}

impl Tool {
    pub fn new(program: String, arguments: Vec<String>) -> Self {
        Self { program, arguments }
    }

    pub fn program(&self) -> &str {
        &self.program
    }

    pub fn arguments(&self) -> &[String] {
        &self.arguments
    }
}

impl Diff {
    pub fn with_report(mut self, report: Report) -> Self {
        self.report = report;
        self
    }

    pub fn with_tool(mut self, tool: Option<Tool>) -> Self {
        self.tool = tool;
        self
    }

    pub fn with_args(mut self, args: Option<Vec<String>>) -> Self {
        self.args = args;
        self
    }

    pub fn merge(&mut self, other: Diff) {
        self.report.merge(other.report);
        self.tool = other.tool.or(self.tool.take());
        self.args = other.args.or(self.args.take());
    }

    pub fn entry(&self) -> Option<&Custom> {
        self.report.entry()
    }

    pub fn summary(&self) -> Option<&Custom> {
        self.report.summary()
    }

    pub fn render(&self) -> Option<&Custom> {
        self.report.render()
    }

    pub fn tool(&self) -> Option<&Tool> {
        self.tool.as_ref()
    }

    pub fn args(&self) -> &[String] {
        self.args.as_deref().unwrap_or_default()
    }
}

impl DiffFile {
    pub fn new(path: PathBuf, system: PathBuf, side: Side, state: DiffState) -> Self {
        Self {
            path,
            system,
            side,
            state,
            content: Vec::new(),
            mode: 0,
            found: None,
            found_mode: None,
        }
    }

    pub fn with_source(mut self, content: Vec<u8>, mode: u32) -> Self {
        self.content = content;
        self.mode = mode;
        self
    }

    pub fn with_system(mut self, content: Vec<u8>, mode: u32) -> Self {
        self.found = Some(content);
        self.found_mode = Some(mode);
        self
    }

    pub fn path(&self) -> &std::path::Path {
        &self.path
    }

    pub fn state(&self) -> DiffState {
        self.state
    }

    pub fn content(&self) -> &[u8] {
        &self.content
    }

    pub fn mode(&self) -> u32 {
        self.mode
    }

    pub fn found(&self) -> Option<&[u8]> {
        self.found.as_deref()
    }

    pub fn found_mode(&self) -> Option<u32> {
        self.found_mode
    }
}

impl DiffCounts {
    pub fn new(side: Side, drifted: usize, total: usize, default: String) -> Self {
        Self {
            side,
            drifted,
            total,
            default,
        }
    }
}

impl IntoLua for &DiffFile {
    fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
        let file = file::table(lua, &self.path, &self.system, self.side, self.state.name())?;

        let content = lua.create_table()?;
        content.set(SOURCE, lua.create_string(&self.content)?)?;
        if let Some(found) = &self.found {
            content.set(SYSTEM, lua.create_string(found)?)?;
        }
        file.set(CONTENT, content)?;

        let mode = lua.create_table()?;
        mode.set(SOURCE, format!("{:04o}", self.mode))?;
        if let Some(found) = self.found_mode {
            mode.set(SYSTEM, format!("{found:04o}"))?;
        }
        file.set(MODE, mode)?;

        Ok(Value::Table(file))
    }
}

impl IntoLua for &DiffCounts {
    fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
        let counts = lua.create_table()?;
        counts.set(SIDE, self.side.dir())?;
        counts.set(DRIFTED, self.drifted)?;
        counts.set(TOTAL, self.total)?;
        counts.set(DEFAULT, self.default.as_str())?;

        Ok(Value::Table(counts))
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    fn file() -> DiffFile {
        DiffFile::new(
            PathBuf::from(".bashrc"),
            PathBuf::from("/home/u/.bashrc"),
            Side::Repository,
            DiffState::Differs,
        )
        .with_source(b"managed\n".to_vec(), 0o644)
        .with_system(b"handwritten\n".to_vec(), 0o600)
    }

    fn read(lua: &Lua, source: &str, value: impl IntoLua) -> String {
        lua.globals().set("subject", value).unwrap();

        lua.load(source).eval().unwrap()
    }

    #[test]
    fn only_the_states_with_content_to_compare_are_staged() {
        assert!(DiffState::Missing.staged());
        assert!(DiffState::Differs.staged());
        assert!(!DiffState::Mode.staged());
        assert!(!DiffState::Other.staged());
    }

    #[test]
    fn a_file_carries_both_sides_into_lua() {
        let lua = Lua::new();
        let file = file();

        assert_eq!(read(&lua, "return subject.path", &file), ".bashrc");
        assert_eq!(
            read(&lua, "return subject.system", &file),
            "/home/u/.bashrc"
        );
        assert_eq!(read(&lua, "return subject.side", &file), "repository");
        assert_eq!(read(&lua, "return subject.state", &file), "differs");
        assert_eq!(
            read(&lua, "return subject.content.source", &file),
            "managed\n"
        );
        assert_eq!(
            read(&lua, "return subject.content.system", &file),
            "handwritten\n"
        );
        assert_eq!(read(&lua, "return subject.mode.source", &file), "0644");
        assert_eq!(read(&lua, "return subject.mode.system", &file), "0600");
    }

    #[test]
    fn a_file_the_system_does_not_hold_carries_one_side_only() {
        let lua = Lua::new();
        let file = DiffFile::new(
            PathBuf::from(".bashrc"),
            PathBuf::from("/home/u/.bashrc"),
            Side::Repository,
            DiffState::Missing,
        )
        .with_source(b"managed\n".to_vec(), 0o644);

        assert_eq!(
            read(&lua, "return tostring(subject.content.system)", &file),
            "nil"
        );
        assert_eq!(
            read(&lua, "return tostring(subject.mode.system)", &file),
            "nil"
        );
    }

    #[test]
    fn the_counts_carry_the_line_they_replace() {
        let lua = Lua::new();
        let counts = DiffCounts::new(
            Side::Generated,
            1,
            12,
            "1 of 12 generated file(s) differ".to_string(),
        );

        assert_eq!(read(&lua, "return subject.side", &counts), "generated");
        assert_eq!(read(&lua, "return subject.drifted .. \"\"", &counts), "1");
        assert_eq!(read(&lua, "return subject.total .. \"\"", &counts), "12");
        assert_eq!(
            read(&lua, "return subject.default", &counts),
            "1 of 12 generated file(s) differ"
        );
    }

    #[test]
    fn a_customization_only_replaces_the_keys_it_carries() {
        let mut diff = Diff::default()
            .with_report(Report::default().with_summary(Some(Custom::Silent)))
            .with_args(Some(vec!["--stat".to_string()]));

        diff.merge(
            Diff::default().with_report(
                Report::default().with_summary(Some(Custom::Text("done".to_string()))),
            ),
        );

        assert!(matches!(diff.summary(), Some(Custom::Text(text)) if text == "done"));
        assert_eq!(diff.args(), ["--stat"]);
        assert!(diff.entry().is_none());
        assert!(diff.tool().is_none());
    }
}
