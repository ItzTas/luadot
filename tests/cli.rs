use std::path::{Path, PathBuf};

use assert_cmd::Command;
use predicates::prelude::*;

fn luadot(home: &Path) -> Command {
    let mut command = Command::cargo_bin("luadot").unwrap();
    command.env_clear().env("HOME", home);
    command
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap()
}

fn only_dir(root: &Path) -> PathBuf {
    let mut dirs: Vec<PathBuf> = std::fs::read_dir(root)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect();

    assert_eq!(dirs.len(), 1);

    dirs.pop().unwrap()
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

    luadot(home.path()).arg("--help").assert().success().stdout(
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
        std::fs::read_to_string(repo.join("home/.vimrc")).unwrap(),
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
fn apply_backs_up_into_the_directory_the_configuration_names_and_restore_finds_it() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    let repo = root.path().join("repo");
    write(&repo.join("home/.bashrc"), "managed\n");
    write(&home.join(".bashrc"), "handwritten\n");
    write(
        &home.join(".config/luadot/config.lua"),
        r#"ld.opt.backup_dir("~/saved")"#,
    );
    write_state(&home, &repo);

    luadot(&home).arg("apply").assert().success();

    let saved = only_dir(&home.join("saved"));
    assert_eq!(read(&saved.join("home/.bashrc")), "handwritten\n");
    assert_eq!(read(&home.join(".bashrc")), "managed\n");
    assert!(!home.join(".local/share/luadot/backups").exists());

    luadot(&home)
        .args(["restore", "--list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("1 file(s)"));

    luadot(&home).args(["restore", "--yes"]).assert().success();
    assert_eq!(read(&home.join(".bashrc")), "handwritten\n");
}

#[test]
fn two_runs_in_a_row_keep_their_own_backup() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    let repo = root.path().join("repo");
    write(&repo.join("home/.bashrc"), "managed\n");
    write(&home.join(".bashrc"), "first\n");
    write_state(&home, &repo);

    luadot(&home).arg("apply").assert().success();
    std::fs::remove_file(home.join(".bashrc")).unwrap();
    write(&home.join(".bashrc"), "second\n");
    luadot(&home).arg("apply").assert().success();

    let mut saved: Vec<String> = std::fs::read_dir(home.join(".local/share/luadot/backups"))
        .unwrap()
        .map(|entry| read(&entry.unwrap().path().join("home/.bashrc")))
        .collect();
    saved.sort();

    assert_eq!(saved, ["first\n", "second\n"]);
}

#[test]
fn the_configuration_points_luadot_at_its_own_repository() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    let repo = root.path().join("dotfiles");
    write(&repo.join("home/.bashrc"), "managed\n");
    write(
        &home.join(".config/luadot/config.lua"),
        &format!("ld.opt.repo_dir({:?})", repo.display()),
    );
    write_state(&home, &root.path().join("gone"));

    luadot(&home).arg("apply").assert().success();

    assert_eq!(read(&home.join(".bashrc")), "managed\n");
}

#[test]
fn clone_takes_the_directory_to_clone_into() {
    let home = tempfile::tempdir().unwrap();

    luadot(home.path())
        .args(["clone", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("[DIR]"));
}

#[test]
fn a_limit_drops_the_oldest_backups() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    let repo = root.path().join("repo");
    write(&repo.join("home/.bashrc"), "managed\n");
    write(
        &home.join(".config/luadot/config.lua"),
        "ld.opt.backup_keep(2)",
    );
    write_state(&home, &repo);

    for contents in ["first\n", "second\n", "third\n"] {
        let _ = std::fs::remove_file(home.join(".bashrc"));
        write(&home.join(".bashrc"), contents);
        luadot(&home).arg("apply").assert().success();
    }

    let mut saved: Vec<String> = std::fs::read_dir(home.join(".local/share/luadot/backups"))
        .unwrap()
        .map(|entry| read(&entry.unwrap().path().join("home/.bashrc")))
        .collect();
    saved.sort();

    assert_eq!(saved, ["second\n", "third\n"]);
}

#[test]
fn rm_backs_up_what_it_takes_out_of_the_repository() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    let repo = home.join(".local/share/luadot/repo");
    write(&repo.join("home/.vimrc"), "set number\n");
    write_state(&home, &repo);

    luadot(&home)
        .args(["rm", "--yes", home.join(".vimrc").to_str().unwrap()])
        .assert()
        .success();

    assert!(!repo.join("home/.vimrc").exists());
    assert_eq!(read(&home.join(".vimrc")), "set number\n");

    let saved = only_dir(&home.join(".local/share/luadot/backups"));
    assert_eq!(
        read(&saved.join("home/.local/share/luadot/repo/home/.vimrc")),
        "set number\n"
    );
}

#[test]
fn apply_places_a_symlink_when_the_configuration_asks_for_one() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    let repo = root.path().join("repo");
    write(&repo.join("home/.bashrc"), "managed\n");
    write(
        &home.join(".config/luadot/config.lua"),
        r#"ld.opt.link("symbolic")"#,
    );
    write_state(&home, &repo);

    luadot(&home).arg("apply").assert().success();

    let placed = home.join(".bashrc");
    let kind = std::fs::symlink_metadata(&placed).unwrap().file_type();
    assert!(kind.is_symlink());
    assert_eq!(
        std::fs::read_link(&placed).unwrap(),
        repo.join("home/.bashrc")
    );
}

#[test]
fn alt_resolves_both_template_forms_and_the_other_commands_walk_past_them() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    let repo = root.path().join("repo");
    write(
        &repo.join("home/.zshrc.luadot/luadot.lua"),
        r#"return ld.alt.expand("zshrc.tmpl.zsh", { editor = "nvim" })"#,
    );
    write(
        &repo.join("home/.zshrc.luadot/zshrc.tmpl.zsh"),
        "export EDITOR=<%= editor %>\n",
    );
    write(
        &repo.join("home/.zprofile.luadot"),
        "<% for _, dir in ipairs({ \"a\", \"b\" }) do -%>\npath+=(<%= dir %>)\n<% end -%>\n",
    );
    write_state(&home, &repo);

    luadot(&home)
        .arg("alt")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "resolved 2 template(s) into 2 file(s)",
        ));
    assert_eq!(
        std::fs::read_to_string(home.join(".zshrc")).unwrap(),
        "export EDITOR=nvim\n"
    );
    assert_eq!(
        std::fs::read_to_string(home.join(".zprofile")).unwrap(),
        "path+=(a)\npath+=(b)\n"
    );

    std::fs::remove_file(home.join(".zshrc")).unwrap();
    std::fs::remove_file(home.join(".zprofile")).unwrap();

    luadot(&home)
        .arg("apply")
        .assert()
        .success()
        .stdout(predicate::str::contains("nothing to apply"));
    luadot(&home)
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("nothing is managed"));
    assert!(!home.join(".zshrc").exists());
    assert!(!home.join(".zprofile").exists());
}

#[test]
fn new_creates_both_template_forms_and_alt_resolves_them() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    let repo = root.path().join("repo");
    std::fs::create_dir_all(&home).unwrap();
    std::fs::create_dir_all(&repo).unwrap();
    write_state(&home, &repo);

    luadot(&home)
        .arg("new")
        .arg(home.join(".config/nvim/init.lua"))
        .assert()
        .success()
        .stdout(predicate::str::contains("created"));
    luadot(&home)
        .args(["new", "-f", "~/.zprofile.luadot"])
        .assert()
        .success();

    assert_eq!(
        read(&repo.join("home/.config/nvim/init.lua.luadot/luadot.lua")),
        "return \"\"\n"
    );
    assert_eq!(read(&repo.join("home/.zprofile.luadot")), "");

    luadot(&home)
        .arg("new")
        .arg(home.join(".zprofile"))
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists"));

    luadot(&home)
        .arg("alt")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "resolved 2 template(s) into 2 file(s)",
        ));

    assert_eq!(read(&home.join(".config/nvim/init.lua")), "");
    assert_eq!(read(&home.join(".zprofile")), "");
}

#[test]
fn a_rule_runs_its_command_once_for_every_file_apply_touched() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    let repo = root.path().join("repo");
    let reloaded = root.path().join("reloaded");
    write(&repo.join("home/.config/mako/config"), "font=monospace\n");
    write(
        &repo.join("home/.config/mako/colors"),
        "background=#000000\n",
    );
    write(&repo.join("home/.bashrc"), "managed\n");
    write(
        &home.join(".config/luadot/config.lua"),
        &format!(
            r#"ld.rules({{ {{ match = "home/.config/mako/**", on_change = "printf x >> {}" }} }})"#,
            reloaded.display()
        ),
    );
    write_state(&home, &repo);

    luadot(&home).arg("apply").assert().success();

    assert_eq!(read(&home.join(".config/mako/config")), "font=monospace\n");
    assert_eq!(read(&reloaded), "x");

    std::fs::remove_file(&reloaded).unwrap();

    luadot(&home).arg("apply").assert().success();
    assert!(!reloaded.exists());
}

#[test]
fn alt_builds_a_file_out_of_fragments_and_runs_the_command_that_follows_it() {
    use std::os::unix::fs::PermissionsExt;

    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    let repo = root.path().join("repo");
    let restarted = root.path().join("restarted");
    write(
        &repo.join("home/.zshrc.luadot/conf.d/20-path.zsh"),
        "path+=(b)\n",
    );
    write(
        &repo.join("home/.zshrc.luadot/conf.d/10-env.zsh"),
        "export A=1\n",
    );
    write(
        &repo.join("home/.zshrc.luadot/luadot.lua"),
        &format!(
            r#"
            local parts = {{}}
            for _, name in ipairs(ld.alt.glob("conf.d/*.zsh")) do
              parts[#parts + 1] = ld.alt.read(name)
            end

            ld.alt.out({{
              content = table.concat(parts, ""),
              mode = "600",
              on_change = "printf ok > {}",
            }})
            "#,
            restarted.display()
        ),
    );
    write_state(&home, &repo);

    luadot(&home).arg("alt").assert().success();

    assert_eq!(read(&home.join(".zshrc")), "export A=1\npath+=(b)\n");
    assert_eq!(
        std::fs::metadata(home.join(".zshrc"))
            .unwrap()
            .permissions()
            .mode()
            & 0o7777,
        0o600
    );
    assert_eq!(read(&restarted), "ok");

    std::fs::remove_file(&restarted).unwrap();

    luadot(&home)
        .arg("alt")
        .assert()
        .success()
        .stdout(predicate::str::contains("1 unchanged"));
    assert!(!restarted.exists());
}

#[test]
fn add_leaves_out_what_the_repository_gitignores() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    let repo = root.path().join("repo");
    gix::init(&repo).unwrap();
    write(&repo.join(".gitignore"), "*.log\nhome/.cache/\n");
    write(&home.join(".config/nvim/init.lua"), "vim.o.number = true\n");
    write(&home.join(".config/nvim/lsp.log"), "noise\n");
    write(&home.join(".cache/state"), "cached\n");
    write_state(&home, &repo);

    luadot(&home)
        .args(["add", home.join(".config/nvim").to_str().unwrap()])
        .assert()
        .success();
    assert!(repo.join("home/.config/nvim/init.lua").exists());
    assert!(!repo.join("home/.config/nvim/lsp.log").exists());

    luadot(&home)
        .args(["add", home.join(".cache").to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::contains(".gitignore"));
    assert!(!repo.join("home/.cache").exists());
}

#[test]
fn add_then_apply_manage_a_system_file_end_to_end() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    let repo = root.path().join("repo");
    let system = root.path().join("etc/app.conf");
    std::fs::create_dir_all(&repo).unwrap();
    write(&system, "conf\n");
    write_state(&home, &repo);

    luadot(&home)
        .args(["add", system.to_str().unwrap()])
        .assert()
        .success();

    let managed = repo.join("root").join(system.strip_prefix("/").unwrap());
    assert_eq!(read(&managed), "conf\n");

    std::fs::remove_file(&system).unwrap();
    luadot(&home).arg("apply").assert().success();
    assert_eq!(read(&system), "conf\n");

    luadot(&home)
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("1 managed file(s) (1 synced"));

    luadot(&home)
        .args(["rm", "--yes", system.to_str().unwrap()])
        .assert()
        .success();
    assert!(!managed.exists());
    assert_eq!(read(&system), "conf\n");
}

#[test]
fn a_mode_rule_lands_on_a_system_file() {
    use std::os::unix::fs::PermissionsExt;

    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    let repo = root.path().join("repo");
    let system = root.path().join("etc/app.conf");
    let managed = repo.join("root").join(system.strip_prefix("/").unwrap());
    write(&managed, "conf\n");
    write(
        &home.join(".config/luadot/config.lua"),
        r#"ld.rules({ match = "root/**", mode = "0640" })"#,
    );
    write_state(&home, &repo);

    luadot(&home).arg("apply").assert().success();

    assert_eq!(read(&system), "conf\n");
    assert_eq!(
        std::fs::metadata(&system).unwrap().permissions().mode() & 0o7777,
        0o640
    );

    luadot(&home)
        .arg("apply")
        .assert()
        .success()
        .stdout(predicate::str::contains("1 unchanged"));
}

#[test]
fn a_replaced_system_file_is_backed_up_and_restored() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    let repo = root.path().join("repo");
    let system = root.path().join("etc/app.conf");
    let managed = repo.join("root").join(system.strip_prefix("/").unwrap());
    write(&managed, "managed\n");
    write(&system, "handwritten\n");
    write_state(&home, &repo);

    luadot(&home).arg("apply").assert().success();
    assert_eq!(read(&system), "managed\n");

    let saved = only_dir(&home.join(".local/share/luadot/backups"));
    let entry = saved.join("root").join(system.strip_prefix("/").unwrap());
    assert_eq!(read(&entry), "handwritten\n");

    luadot(&home).args(["restore", "--yes"]).assert().success();
    assert_eq!(read(&system), "handwritten\n");
}

#[test]
fn diff_reports_the_drift_and_what_the_system_is_missing() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    let repo = root.path().join("repo");
    write(&repo.join("home/.bashrc"), "managed\n");
    write(&repo.join("home/.vimrc"), "set number\n");
    write(&repo.join("home/.zshrc"), "synced\n");
    write(&home.join(".bashrc"), "handwritten\n");
    write(&home.join(".zshrc"), "synced\n");
    write_state(&home, &repo);

    luadot(&home).arg("diff").assert().success().stdout(
        predicate::str::contains("repository/home/.bashrc system/home/.bashrc")
            .and(predicate::str::contains("-managed"))
            .and(predicate::str::contains("+handwritten"))
            .and(predicate::str::contains("repository/home/.vimrc"))
            .and(predicate::str::contains("-set number"))
            .and(predicate::str::contains(".zshrc").not())
            .and(predicate::str::contains("2 of 3 managed file(s) differ")),
    );
}

#[test]
fn diff_narrows_to_the_path_it_is_given_and_leaves_nothing_behind() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    let repo = root.path().join("repo");
    write(&repo.join("home/.bashrc"), "managed\n");
    write(&repo.join("home/.vimrc"), "set number\n");
    write(&home.join(".bashrc"), "handwritten\n");
    write_state(&home, &repo);

    let before = mirrors();

    luadot(&home)
        .args(["diff", home.join(".bashrc").to_str().unwrap()])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("+handwritten")
                .and(predicate::str::contains(".vimrc").not())
                .and(predicate::str::contains("1 of 1 managed file(s) differ")),
        );

    assert_eq!(mirrors(), before);
}

#[test]
fn diff_reports_a_system_file_whose_mode_is_all_that_drifted() {
    use std::os::unix::fs::PermissionsExt;

    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    let repo = root.path().join("repo");
    let system = root.path().join("etc/app.conf");
    let managed = repo.join("root").join(system.strip_prefix("/").unwrap());
    write(&managed, "conf\n");
    write(&system, "conf\n");
    std::fs::set_permissions(&system, std::fs::Permissions::from_mode(0o600)).unwrap();
    write(
        &home.join(".config/luadot/config.lua"),
        r#"ld.rules({ match = "root/**", mode = "0640" })"#,
    );
    write_state(&home, &repo);

    luadot(&home).arg("diff").assert().success().stdout(
        predicate::str::contains("mode")
            .and(predicate::str::contains("0600 -> 0640"))
            .and(predicate::str::contains("1 of 1 managed file(s) differ")),
    );
}

fn mirrors() -> usize {
    std::fs::read_dir(std::env::temp_dir())
        .unwrap()
        .filter(|entry| {
            entry
                .as_ref()
                .unwrap()
                .file_name()
                .to_string_lossy()
                .starts_with("luadot-diff-")
        })
        .count()
}
