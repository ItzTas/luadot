use std::path::Path;

use assert_cmd::Command;
use predicates::prelude::*;

fn luadot(home: &Path) -> Command {
    let mut command = Command::cargo_bin("luadot").unwrap();
    command.env_clear().env("HOME", home);
    command
}

fn write(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(path, contents).unwrap();
}

fn write_state(home: &Path, repo: &Path) {
    write(
        &home.join(".local/share/luadot/state.json"),
        &format!(r#"{{"repo":{:?},"classes":{{}}}}"#, repo.display()),
    );
}

#[test]
fn version_prints_the_package_version() {
    let home = tempfile::tempdir().unwrap();

    luadot(home.path())
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn help_lists_the_commands() {
    let home = tempfile::tempdir().unwrap();

    luadot(home.path())
        .arg("--help")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Usage: luadot")
                .and(predicate::str::contains("apply"))
                .and(predicate::str::contains("restore"))
                .and(predicate::str::contains("completions")),
        );
}

#[test]
fn a_command_explains_itself() {
    let home = tempfile::tempdir().unwrap();

    luadot(home.path())
        .args(["rm", "--help"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Usage: luadot rm")
                .and(predicate::str::contains("--yes"))
                .and(predicate::str::contains("--dry-run")),
        );
}

#[test]
fn a_bare_invocation_prints_the_help() {
    let home = tempfile::tempdir().unwrap();

    luadot(home.path())
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("Usage: luadot"));
}

#[test]
fn an_unknown_command_is_refused() {
    let home = tempfile::tempdir().unwrap();

    luadot(home.path())
        .arg("nope")
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("unrecognized subcommand 'nope'"));
}

#[test]
fn an_unknown_option_is_refused() {
    let home = tempfile::tempdir().unwrap();

    luadot(home.path())
        .args(["rm", "--force", ".bashrc"])
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("unexpected argument '--force'"));
}

#[test]
fn a_command_without_a_repository_says_how_to_get_one() {
    let home = tempfile::tempdir().unwrap();

    luadot(home.path())
        .arg("status")
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains(
            "status: no repository set; run `luadot clone <url>` first",
        ));
}

#[test]
fn exec_runs_lua_from_a_source_string() {
    let home = tempfile::tempdir().unwrap();

    luadot(home.path())
        .args(["exec", r#"print("hi from lua")"#])
        .assert()
        .success()
        .stdout(predicate::str::contains("hi from lua"));
}

#[test]
fn exec_keeps_the_flags_after_the_target_for_the_script() {
    let home = tempfile::tempdir().unwrap();

    luadot(home.path())
        .args(["exec", "print(ld.argv.name, ld.argv.args[2])", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("exec").and(predicate::str::contains("--json")));
}

#[test]
fn add_then_apply_manage_a_file_end_to_end() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    let repo = root.path().join("repo");
    std::fs::create_dir_all(&repo).unwrap();
    write(&home.join(".vimrc"), "set number\n");
    write_state(&home, &repo);

    luadot(&home)
        .args(["add", home.join(".vimrc").to_str().unwrap()])
        .assert()
        .success();
    assert_eq!(
        std::fs::read_to_string(repo.join(".vimrc")).unwrap(),
        "set number\n"
    );

    std::fs::remove_file(home.join(".vimrc")).unwrap();
    luadot(&home).arg("apply").assert().success();
    assert_eq!(
        std::fs::read_to_string(home.join(".vimrc")).unwrap(),
        "set number\n"
    );

    luadot(&home)
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("1 managed file(s)"));
}

#[test]
fn apply_places_a_symlink_when_the_configuration_asks_for_one() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    let repo = root.path().join("repo");
    write(&repo.join(".bashrc"), "managed\n");
    write(
        &home.join(".config/luadot/ld.lua"),
        r#"ld.opt.link("symbolic")"#,
    );
    write_state(&home, &repo);

    luadot(&home).arg("apply").assert().success();

    let placed = home.join(".bashrc");
    let kind = std::fs::symlink_metadata(&placed).unwrap().file_type();
    assert!(kind.is_symlink());
    assert_eq!(std::fs::read_link(&placed).unwrap(), repo.join(".bashrc"));
}
