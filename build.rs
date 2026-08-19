use std::env;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::process::Command;

use sha2::{Digest, Sha256};

const LPEG_VERSION: &str = "1.1.0";
const LPEG_URL: &str = "https://github.com/roberto-ieru/LPeg/archive/refs/tags/v1.1.0.tar.gz";
const LPEG_SHA256: &str = "89bcd56c6cb7d001c12fd2c0c486c06ca8283f8b58986f593d62cc26e1458de4";
const LPEG_SOURCES: [&str; 6] = [
    "lpcap.c",
    "lpcode.c",
    "lpcset.c",
    "lpprint.c",
    "lptree.c",
    "lpvm.c",
];

fn main() {
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=.githooks");

    install_hooks();

    if let Err(err) = build_lpeg() {
        panic!("failed to build lpeg {LPEG_VERSION}: {err}");
    }
}

fn install_hooks() {
    let inside_work_tree = Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .output()
        .map(|out| out.status.success() && out.stdout.starts_with(b"true"))
        .unwrap_or(false);

    if !inside_work_tree {
        return;
    }

    let _ = Command::new("git")
        .args(["config", "core.hooksPath", ".githooks"])
        .status();
}

fn build_lpeg() -> Result<(), Box<dyn Error>> {
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    let source = fetch_lpeg(&out_dir)?;
    let lua = lua_src::Build::new().build(lua_src::Lua54);

    let mut build = cc::Build::new();
    build.include(lua.include_dir()).include(&source);
    for file in LPEG_SOURCES {
        build.file(source.join(file));
    }
    build.compile("lpeg");

    let re = source.join("re.lua");
    println!("cargo::rustc-env=LPEG_RE_PATH={}", re.display());

    Ok(())
}

fn fetch_lpeg(out_dir: &Path) -> Result<PathBuf, Box<dyn Error>> {
    let source = out_dir.join(format!("LPeg-{LPEG_VERSION}"));
    if source.join("re.lua").is_file() {
        return Ok(source);
    }

    let tarball = ureq::get(LPEG_URL).call()?.into_body().read_to_vec()?;
    verify(&tarball, LPEG_SHA256)?;

    let decoder = flate2::read::GzDecoder::new(tarball.as_slice());
    tar::Archive::new(decoder).unpack(out_dir)?;

    Ok(source)
}

fn verify(bytes: &[u8], expected: &str) -> Result<(), Box<dyn Error>> {
    let digest = Sha256::digest(bytes);
    let digest: String = digest.iter().map(|byte| format!("{byte:02x}")).collect();
    if digest != expected {
        return Err(format!("checksum mismatch: expected {expected}, got {digest}").into());
    }

    Ok(())
}
