use std::path::{Path, PathBuf};

use super::constants::TEMPLATE_SUFFIX;

pub fn is_template(path: &Path) -> bool {
    template_name(path).is_some()
}

pub fn template_target(dir: &Path) -> Option<PathBuf> {
    Some(dir.with_file_name(template_name(dir)?))
}

pub fn template_dir(target: &Path) -> Option<PathBuf> {
    let name = target.file_name()?.to_str()?;

    Some(target.with_file_name(format!("{name}{TEMPLATE_SUFFIX}")))
}

fn template_name(path: &Path) -> Option<&str> {
    let name = path.file_name()?.to_str()?;
    let target = name.strip_suffix(TEMPLATE_SUFFIX)?;

    if target.is_empty() {
        return None;
    }

    Some(target)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_directory_carrying_the_suffix_is_a_template() {
        assert!(is_template(Path::new("/repo/.zshrc.luadot")));
        assert!(is_template(Path::new("/repo/.config/nvim/init.lua.luadot")));
    }

    #[test]
    fn the_target_drops_the_suffix_and_keeps_the_location() {
        assert_eq!(
            template_target(Path::new("/repo/.config/nvim/init.lua.luadot")),
            Some(PathBuf::from("/repo/.config/nvim/init.lua"))
        );
    }
}
