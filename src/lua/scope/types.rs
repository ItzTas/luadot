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
pub struct Scope {
    dir: PathBuf,
    home: PathBuf,
    dest: Option<PathBuf>,
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

impl Scope {
    pub fn new(dir: PathBuf, home: PathBuf) -> Self {
        Self {
            dir,
            home,
            dest: None,
            outputs: Vec::new(),
        }
    }

    pub fn with_dest(mut self, dest: PathBuf) -> Self {
        self.dest = Some(dest);
        self
    }

    pub fn building(lua: &Lua) -> mlua::Result<AppDataRefMut<'_, Scope>> {
        lua.app_data_mut::<Scope>()
            .ok_or_else(|| mlua::Error::external("the scope is not available"))
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

    pub fn destination(&self, raw: Option<&str>) -> Option<PathBuf> {
        let Some(raw) = raw else {
            return self.dest.clone();
        };

        Some(utils::expand(&self.home, Path::new(raw)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scope(dir: &Path) -> Scope {
        Scope::new(dir.to_path_buf(), PathBuf::from("/home/u"))
            .with_dest(PathBuf::from("/home/u/.zshrc"))
    }

    #[test]
    fn resolve_rejects_what_is_not_a_file() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join("variants")).unwrap();

        let scope = scope(dir.path());

        assert_eq!(scope.resolve("missing.zsh"), None);
        assert_eq!(scope.resolve("variants"), None);
        assert_eq!(scope.resolve("/nowhere/missing.zsh"), None);
    }

    #[test]
    fn a_scope_that_mirrors_nothing_has_no_default_destination() {
        let scope = Scope::new(
            PathBuf::from("/home/u/.config/luadot"),
            PathBuf::from("/home/u"),
        );

        assert_eq!(scope.destination(None), None);
        assert_eq!(
            scope.destination(Some("~/.netrc")),
            Some(PathBuf::from("/home/u/.netrc"))
        );
    }

    #[test]
    fn destination_expands_a_declared_path() {
        let scope = scope(Path::new("/repo/.zshrc.luadot"));

        assert_eq!(
            scope.destination(Some("~/.config/zsh/.zshrc")),
            Some(PathBuf::from("/home/u/.config/zsh/.zshrc"))
        );
        assert_eq!(
            scope.destination(Some(".config/zsh/.zshrc")),
            Some(PathBuf::from("/home/u/.config/zsh/.zshrc"))
        );
        assert_eq!(
            scope.destination(Some("/etc/zsh/zshrc")),
            Some(PathBuf::from("/etc/zsh/zshrc"))
        );
    }
}
