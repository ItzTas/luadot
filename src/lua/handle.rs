use std::path::{Path, PathBuf};

use mlua::{UserData, UserDataFields};

#[derive(Clone, Debug)]
pub struct FileHandle {
    path: PathBuf,
}

impl FileHandle {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl UserData for FileHandle {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("path", |_, this| {
            Ok(this.path.to_string_lossy().into_owned())
        });
    }
}
