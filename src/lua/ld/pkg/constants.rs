pub const NAMESPACE: &str = "pkg";

pub const INSTALL: &str = "install";

pub const SUDO: &str = "sudo";

pub const MANAGERS: [(&str, &[&str]); 3] = [
    ("pacman", &["-S", "--needed", "--noconfirm"]),
    ("apt-get", &["install", "-y"]),
    ("dnf", &["install", "-y"]),
];
