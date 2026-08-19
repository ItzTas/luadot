use std::path::{Path, PathBuf};

use mlua::{AppDataRefMut, Lua, UserData};

use crate::files::{ConflictPolicy, LinkMode};
use crate::utils;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Handle(PathBuf);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Content {
    File(PathBuf),
    Text(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Output {
    dest: PathBuf,
    content: Content,
    link: Option<LinkMode>,
    conflict: Option<ConflictPolicy>,
    mode: Option<u32>,
    on_change: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Template {
    dir: PathBuf,
    home: PathBuf,
    dest: PathBuf,
    outputs: Vec<Output>,
}

impl Handle {
    pub fn new(path: PathBuf) -> Self {
        Self(path)
    }

    pub fn path(&self) -> &Path {
        &self.0
    }
}

impl UserData for Handle {}

impl Output {
    pub fn new(
        dest: PathBuf,
        content: Content,
        link: Option<LinkMode>,
        conflict: Option<ConflictPolicy>,
    ) -> Self {
        Self {
            dest,
            content,
            link,
            conflict,
            mode: None,
            on_change: None,
        }
    }

    pub fn with_mode(mut self, mode: Option<u32>) -> Self {
        self.mode = mode;
        self
    }

    pub fn with_on_change(mut self, on_change: Option<String>) -> Self {
        self.on_change = on_change;
        self
    }

    pub fn dest(&self) -> &Path {
        &self.dest
    }

    pub fn content(&self) -> &Content {
        &self.content
    }

    pub fn link(&self) -> Option<LinkMode> {
        self.link
    }

    pub fn conflict(&self) -> Option<ConflictPolicy> {
        self.conflict
    }

    pub fn mode(&self) -> Option<u32> {
        self.mode
    }

    pub fn on_change(&self) -> Option<&str> {
        self.on_change.as_deref()
    }
}

impl Template {
    pub fn new(dir: PathBuf, home: PathBuf, dest: PathBuf) -> Self {
        Self {
            dir,
            home,
            dest,
            outputs: Vec::new(),
        }
    }

    pub fn building(lua: &Lua) -> mlua::Result<AppDataRefMut<'_, Template>> {
        lua.app_data_mut::<Template>()
            .ok_or_else(|| mlua::Error::external("the template is not available"))
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    pub fn home(&self) -> &Path {
        &self.home
    }

    pub fn add_output(&mut self, output: Output) {
        self.outputs.push(output);
    }

    pub fn into_outputs(self) -> Vec<Output> {
        self.outputs
    }

    pub fn resolve(&self, name: &str) -> Option<PathBuf> {
        let path = self.dir.join(name);
        path.is_file().then_some(path)
    }

    pub fn destination(&self, raw: Option<&str>) -> PathBuf {
        let Some(raw) = raw else {
            return self.dest.clone();
        };

        utils::expand(&self.home, Path::new(raw))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn template(dir: &Path) -> Template {
        Template::new(
            dir.to_path_buf(),
            PathBuf::from("/home/u"),
            PathBuf::from("/home/u/.zshrc"),
        )
    }

    #[test]
    fn resolve_finds_a_file_inside_the_template() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join("variants")).unwrap();
        std::fs::write(dir.path().join("variants/laptop.zsh"), "data").unwrap();

        assert_eq!(
            template(dir.path()).resolve("variants/laptop.zsh"),
            Some(dir.path().join("variants/laptop.zsh"))
        );
    }

    #[test]
    fn resolve_rejects_what_is_not_a_file() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join("variants")).unwrap();

        let template = template(dir.path());

        assert_eq!(template.resolve("missing.zsh"), None);
        assert_eq!(template.resolve("variants"), None);
        assert_eq!(template.resolve("/nowhere/missing.zsh"), None);
    }

    #[test]
    fn resolve_reaches_outside_the_template() {
        let root = tempfile::tempdir().unwrap();
        let dir = root.path().join(".zshrc.luadot");
        std::fs::create_dir(&dir).unwrap();
        let shared = root.path().join("shared.zsh");
        std::fs::write(&shared, "data").unwrap();

        let template = template(&dir);

        assert_eq!(
            template.resolve("../shared.zsh"),
            Some(dir.join("../shared.zsh"))
        );
        assert_eq!(
            template.resolve(&shared.display().to_string()),
            Some(shared)
        );
    }

    #[test]
    fn destination_defaults_to_the_mirrored_path() {
        assert_eq!(
            template(Path::new("/repo/.zshrc.luadot")).destination(None),
            PathBuf::from("/home/u/.zshrc")
        );
    }

    #[test]
    fn destination_expands_a_declared_path() {
        let template = template(Path::new("/repo/.zshrc.luadot"));

        assert_eq!(
            template.destination(Some("~/.config/zsh/.zshrc")),
            PathBuf::from("/home/u/.config/zsh/.zshrc")
        );
        assert_eq!(
            template.destination(Some(".config/zsh/.zshrc")),
            PathBuf::from("/home/u/.config/zsh/.zshrc")
        );
        assert_eq!(
            template.destination(Some("/etc/zsh/zshrc")),
            PathBuf::from("/etc/zsh/zshrc")
        );
    }
}
