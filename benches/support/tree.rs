use std::path::{Path, PathBuf};

use super::constants::{BODY_LINES, GIT_OBJECTS, TEMPLATE_COUNT};

pub fn write(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("a parent directory");
    }
    std::fs::write(path, contents).expect("a written file");
}

pub fn fill(root: &Path, count: usize) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = (0..count)
        .map(|index| {
            let path = root.join(managed_name(index));
            write(&path, &body(index));
            path
        })
        .collect();
    files.sort();
    files
}

pub fn git_noise(root: &Path) {
    write(&root.join(".git/config"), "[core]\n");
    for index in 0..GIT_OBJECTS {
        write(
            &root.join(format!(".git/objects/{:02x}/{index:08x}", index % 256)),
            "packed",
        );
    }
}

pub fn templates(root: &Path) -> Vec<PathBuf> {
    (0..TEMPLATE_COUNT)
        .map(|index| {
            let dir = root.join(format!(".config/tool{index:02}/init.conf.luadot"));
            write(
                &dir.join("luadot.lua"),
                "return ld.alt.file(\"variant.conf\")\n",
            );
            write(&dir.join("variant.conf"), "variant\n");
            dir
        })
        .collect()
}

pub fn managed_name(index: usize) -> PathBuf {
    let group = index / 8;
    match index % 4 {
        0 => PathBuf::from(format!(".file{index:04}rc")),
        1 => PathBuf::from(format!(".config/app{group:03}/config{index:04}.toml")),
        2 => PathBuf::from(format!(".config/app{group:03}/themes/theme{index:04}.toml")),
        _ => PathBuf::from(format!(".local/share/app{group:03}/data{index:04}.json")),
    }
}

fn body(index: usize) -> String {
    (0..BODY_LINES)
        .map(|line| format!("setting{line} = \"value {index}\"\n"))
        .collect()
}
