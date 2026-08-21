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

fn luadot_with_git(home: &Path) -> Command {
    let mut command = luadot(home);
    command.env("PATH", std::env::var("PATH").unwrap_or_default());
    command
}

fn git(repo: &Path, args: &[&str]) -> String {
    let output = std::process::Command::new("git")
        .current_dir(repo)
        .args(args)
        .output()
        .unwrap();
    assert!(output.status.success(), "git {args:?} failed");

    String::from_utf8(output.stdout).unwrap()
}

fn repository(repo: &Path) {
    std::fs::create_dir_all(repo).unwrap();

    for args in [
        vec!["init", "--quiet"],
        vec!["config", "user.email", "test@luadot"],
        vec!["config", "user.name", "luadot"],
        vec!["config", "commit.gpgsign", "false"],
    ] {
        git(repo, &args);
    }
}

fn staged(repo: &Path) -> String {
    git(repo, &["diff", "--cached", "--name-only"])
}

fn subject(repo: &Path) -> String {
    git(repo, &["log", "-1", "--format=%s"])
}

fn write_state(home: &Path, repo: &Path) {
    write(
        &home.join(".local/share/luadot/state.json"),
        &format!(r#"{{"repo":{:?},"classes":{{}}}}"#, repo.display()),
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
fn a_command_without_a_repository_says_how_to_get_one() {
    let home = tempfile::tempdir().unwrap();

    luadot(home.path())
        .arg("status")
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains(
            "status: no repository set; run `luadot clone <url>` or `luadot init` first",
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
        .stdout(predicate::str::contains(
            "nothing to apply, every managed file is synced",
        ));
}

#[test]
fn add_stages_what_it_mirrors_and_rm_stages_what_it_takes_out() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    let repo = root.path().join("repo");
    repository(&repo);
    write(&home.join(".vimrc"), "set number\n");
    write_state(&home, &repo);

    luadot_with_git(&home)
        .args(["add", home.join(".vimrc").to_str().unwrap()])
        .assert()
        .success();
    assert_eq!(staged(&repo), "home/.vimrc\n");

    git(&repo, &["commit", "--quiet", "-m", "first"]);
    luadot_with_git(&home)
        .args(["rm", "-y", home.join(".vimrc").to_str().unwrap()])
        .assert()
        .success();

    assert_eq!(staged(&repo), "home/.vimrc\n");
    assert_eq!(read(&home.join(".vimrc")), "set number\n");
}

#[test]
fn sync_commits_what_the_repository_holds() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    let repo = root.path().join("repo");
    repository(&repo);
    write(&home.join(".vimrc"), "set number\n");
    write_state(&home, &repo);

    luadot_with_git(&home)
        .args(["add", home.join(".vimrc").to_str().unwrap()])
        .assert()
        .success();
    luadot_with_git(&home)
        .args(["sync", "--no-push", "-m", "first"])
        .assert()
        .success();

    assert_eq!(subject(&repo), "first\n");
    assert_eq!(staged(&repo), "");

    luadot_with_git(&home)
        .args(["sync", "--no-push"])
        .assert()
        .success()
        .stdout(predicate::str::contains("nothing to commit"));
}

#[test]
fn sync_without_a_git_repository_says_how_to_get_one() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    let repo = root.path().join("repo");
    std::fs::create_dir_all(&repo).unwrap();
    write_state(&home, &repo);

    luadot_with_git(&home)
        .args(["sync"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("is not a git repository"));
}

#[test]
fn a_file_the_configuration_writes_waits_for_a_run_that_is_not_dry() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    let repo = root.path().join("repo");
    let generated = home.join(".config/mako/config");
    write(&repo.join("home/.bashrc"), "managed\n");
    write(
        &home.join(".config/luadot/config.lua"),
        r#"ld.alt.out({ dest = "~/.config/mako/config", content = "font=monospace\n" })"#,
    );
    write_state(&home, &repo);

    luadot(&home).args(["apply", "--dry-run"]).assert().success();
    assert!(!generated.exists());

    luadot(&home).arg("apply").assert().success();
    assert_eq!(read(&generated), "font=monospace\n");
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
fn init_creates_a_repository_and_makes_it_the_managed_one() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    let repo = root.path().join("dotfiles");
    write(&home.join(".vimrc"), "set number\n");

    luadot(&home)
        .args(["init", repo.to_str().unwrap()])
        .assert()
        .success();
    assert!(repo.join(".git").is_dir());

    luadot(&home)
        .args(["add", home.join(".vimrc").to_str().unwrap()])
        .assert()
        .success();
    assert_eq!(read(&repo.join("home/.vimrc")), "set number\n");
}

#[test]
fn init_refuses_a_directory_holding_something() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    let repo = root.path().join("dotfiles");
    write(&repo.join("kept.txt"), "data");

    luadot(&home)
        .args(["init", repo.to_str().unwrap()])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("init: destination"));
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
fn alt_resolves_both_template_forms_and_apply_walks_past_them() {
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
        .args(["tmpl", "alt"])
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
        .stdout(predicate::str::contains("2 template(s) not resolved"));
    luadot(&home)
        .args(["status", "--templates"])
        .assert()
        .success()
        .stdout(predicate::str::contains("2 template(s) into 2 file(s)"));
    assert!(!home.join(".zshrc").exists());
    assert!(!home.join(".zprofile").exists());
}

#[test]
fn tmpl_new_creates_both_template_forms_and_tmpl_alt_resolves_them() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    let repo = root.path().join("repo");
    std::fs::create_dir_all(&home).unwrap();
    std::fs::create_dir_all(&repo).unwrap();
    write_state(&home, &repo);

    luadot(&home)
        .args(["tmpl", "new"])
        .arg(home.join(".config/nvim/init.lua"))
        .assert()
        .success()
        .stdout(predicate::str::contains("created"))
        .stdout(predicate::str::contains(
            "added home/.config/nvim/init.lua.luadot",
        ));
    luadot(&home)
        .args(["tmpl", "new", "-f", "~/.zprofile.luadot"])
        .assert()
        .success();

    assert_eq!(
        read(&home.join(".config/nvim/init.lua.luadot/luadot.lua")),
        "return \"\"\n"
    );
    assert_eq!(
        read(&repo.join("home/.config/nvim/init.lua.luadot/luadot.lua")),
        "return \"\"\n"
    );
    assert_eq!(read(&home.join(".zprofile.luadot")), "");
    assert_eq!(read(&repo.join("home/.zprofile.luadot")), "");

    luadot(&home)
        .args(["tmpl", "new"])
        .arg(home.join(".zprofile"))
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists"));

    luadot(&home)
        .args(["tmpl", "alt"])
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

    luadot(&home).args(["tmpl", "alt"]).assert().success();

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
        .args(["tmpl", "alt"])
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
        .stdout(predicate::str::contains(
            "nothing to apply, every managed file is synced",
        ));

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
        predicate::str::contains("diff --git a/home/.bashrc b/home/.bashrc")
            .and(predicate::str::contains("--- a/home/.bashrc"))
            .and(predicate::str::contains("+++ b/home/.bashrc"))
            .and(predicate::str::contains("-managed"))
            .and(predicate::str::contains("+handwritten"))
            .and(predicate::str::contains(
                "diff --git a/home/.vimrc b/home/.vimrc",
            ))
            .and(predicate::str::contains("deleted file mode 100644"))
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

    let temp = root.path().join("temp");
    std::fs::create_dir_all(&temp).unwrap();

    luadot(&home)
        .env("TMPDIR", &temp)
        .args(["diff", home.join(".bashrc").to_str().unwrap()])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("+handwritten")
                .and(predicate::str::contains(".vimrc").not())
                .and(predicate::str::contains("1 of 1 managed file(s) differ")),
        );

    assert_eq!(mirrors(&temp), 0);
}

#[test]
fn status_groups_what_apply_would_touch_under_the_repository_it_reads() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    let repo = root.path().join("repo");
    write(&repo.join("home/.bashrc"), "managed\n");
    write(&repo.join("home/.vimrc"), "set number\n");
    write(&home.join(".vimrc"), "set number\n");
    write(&repo.join("home/.zshrc"), "repository\n");
    write(&home.join(".zshrc"), "system\n");
    write_state(&home, &repo);

    luadot(&home).arg("status").assert().success().stdout(
        predicate::str::contains(format!("On repository {}", repo.display()))
            .and(predicate::str::contains("3 managed file(s)"))
            .and(predicate::str::contains("Files not on the system:"))
            .and(predicate::str::contains(
                "(use \"luadot apply <path>...\" to write them)",
            ))
            .and(predicate::str::contains(
                "        missing:     home/.bashrc",
            ))
            .and(predicate::str::contains("Files not linked:"))
            .and(predicate::str::contains("        unlinked:    home/.vimrc"))
            .and(predicate::str::contains("Files that differ:"))
            .and(predicate::str::contains(
                "(use \"luadot diff <path>...\" to see what changed)",
            ))
            .and(predicate::str::contains("        differs:     home/.zshrc")),
    );
}

#[test]
fn status_writes_the_entries_and_the_summary_the_configuration_asks_for() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    let repo = root.path().join("repo");
    write(&repo.join("home/.bashrc"), "managed\n");
    write(&repo.join("home/.vimrc"), "set number\n");
    write(&home.join(".vimrc"), "set number\n");
    write(
        &home.join(".config/luadot/config.lua"),
        r#"
        ld.on.status({
          entry = function(file)
            return "» " .. file.state .. " " .. file.path
          end,
          summary = function(counts)
            return counts.synced .. "/" .. counts.total .. " on the " .. counts.side
          end,
        })
        "#,
    );
    write_state(&home, &repo);

    luadot(&home).arg("status").assert().success().stdout(
        predicate::str::contains("» missing home/.bashrc")
            .and(predicate::str::contains("» unlinked home/.vimrc"))
            .and(predicate::str::contains("0/2 on the repository"))
            .and(predicate::str::contains("managed file(s) (").not()),
    );
}

#[test]
fn status_hands_every_file_it_inspected_to_the_configuration() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    let repo = root.path().join("repo");
    write(&repo.join("home/.bashrc"), "managed\n");
    write(&repo.join("home/.vimrc"), "set number\n");
    write(&home.join(".vimrc"), "set number\n");
    write(&repo.join("home/.zshrc"), "synced\n");
    std::fs::hard_link(repo.join("home/.zshrc"), home.join(".zshrc")).unwrap();
    write(
        &home.join(".config/luadot/config.lua"),
        r#"
        ld.on.status({
          render = function(files)
            for _, file in ipairs(files) do
              ld.print.entry(file.state, file.path, { tone = "muted" })
            end
            ld.print("counted " .. #files)
          end,
          summary = false,
        })
        "#,
    );
    write_state(&home, &repo);

    luadot(&home).arg("status").assert().success().stdout(
        predicate::str::contains("missing    home/.bashrc")
            .and(predicate::str::contains("unlinked   home/.vimrc"))
            .and(predicate::str::contains("synced     home/.zshrc"))
            .and(predicate::str::contains("counted 3"))
            .and(predicate::str::contains("managed file(s)").not()),
    );
}

#[test]
fn diff_hands_every_drifted_file_to_the_configuration_and_runs_no_diff_of_its_own() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    let repo = root.path().join("repo");
    write(&repo.join("home/.bashrc"), "managed\n");
    write(&home.join(".bashrc"), "handwritten\n");
    write(
        &home.join(".config/luadot/config.lua"),
        r#"
        ld.on.diff({
          render = function(files)
            for _, file in ipairs(files) do
              ld.print.section(file.path)
              ld.print(file.content.source, { mark = "-", newline = false })
              ld.print(file.content.system, { mark = "+", newline = false })
            end
          end,
          summary = false,
        })
        "#,
    );
    write_state(&home, &repo);

    luadot(&home).arg("diff").assert().success().stdout(
        predicate::str::contains("home/.bashrc")
            .and(predicate::str::contains("- managed"))
            .and(predicate::str::contains("+ handwritten"))
            .and(predicate::str::contains("diff --git").not())
            .and(predicate::str::contains("managed file(s) differ").not()),
    );
}

#[test]
fn diff_runs_the_tool_the_configuration_names_over_the_two_sides() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    let repo = root.path().join("repo");
    write(&repo.join("home/.bashrc"), "managed\n");
    write(&home.join(".bashrc"), "handwritten\n");
    write(
        &home.join(".config/luadot/config.lua"),
        r#"ld.on.diff({ tool = { "echo", "compared" } })"#,
    );
    write_state(&home, &repo);

    luadot(&home).arg("diff").assert().success().stdout(
        predicate::str::contains("compared repository system")
            .and(predicate::str::contains("diff --git").not())
            .and(predicate::str::contains("1 of 1 managed file(s) differ")),
    );
}

#[test]
fn diff_reports_a_customization_that_breaks() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    let repo = root.path().join("repo");
    write(&repo.join("home/.bashrc"), "managed\n");
    write(&home.join(".bashrc"), "handwritten\n");
    write(
        &home.join(".config/luadot/config.lua"),
        r#"ld.on.diff({ summary = function() return 1 end })"#,
    );
    write_state(&home, &repo);

    luadot(&home)
        .arg("diff")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "diff: `ld.on.diff`: `summary` returned integer",
        ));
}

#[test]
fn print_writes_the_line_the_way_the_script_asks_for_it() {
    let home = tempfile::tempdir().unwrap();

    luadot(home.path())
        .args([
            "exec",
            r#"
            ld.print.section("Repository")
            ld.print.field("path", "/data/repo")
            ld.print.entry("create", "~/.bashrc", { tone = "good" })
            ld.print("done", { mark = "»", indent = 2 })
            ld.print.note("nothing is managed")
            ld.print.warn("careful")
            "#,
        ])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Repository")
                .and(predicate::str::contains("path        /data/repo"))
                .and(predicate::str::contains("create     ~/.bashrc"))
                .and(predicate::str::contains("  » done"))
                .and(predicate::str::contains("luadot: nothing is managed")),
        )
        .stderr(predicate::str::contains("luadot: careful"));
}

#[test]
fn rm_takes_a_template_out_and_leaves_the_file_it_produced_working() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    let repo = root.path().join("repo");
    write(&repo.join("home/.zshrc.luadot/laptop.zsh"), "laptop\n");
    write(
        &repo.join("home/.zshrc.luadot/luadot.lua"),
        r#"return { content = ld.alt.file("laptop.zsh"), link = "symbolic" }"#,
    );
    write_state(&home, &repo);

    luadot(&home).args(["tmpl", "alt"]).assert().success();
    assert_eq!(
        std::fs::read_link(home.join(".zshrc")).unwrap(),
        repo.join("home/.zshrc.luadot/laptop.zsh")
    );

    luadot(&home)
        .args(["rm", "--dry-run", home.join(".zshrc").to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "would stop managing 1 template(s) (1 restored, 0 left untouched)",
        ));
    assert!(repo.join("home/.zshrc.luadot").is_dir());

    luadot(&home)
        .args(["rm", "--yes", home.join(".zshrc").to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "stopped managing 1 template(s) (1 restored, 0 left untouched)",
        ));

    assert!(!repo.join("home/.zshrc.luadot").exists());
    assert!(
        !std::fs::symlink_metadata(home.join(".zshrc"))
            .unwrap()
            .is_symlink()
    );
    assert_eq!(read(&home.join(".zshrc")), "laptop\n");
}

#[test]
fn diff_shows_what_a_template_would_write_only_when_it_is_asked_to() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    let repo = root.path().join("repo");
    write(&repo.join("home/.bashrc"), "managed\n");
    write(&home.join(".bashrc"), "managed\n");
    write(
        &repo.join("home/.zshrc.luadot/luadot.lua"),
        r#"return "export EDITOR=nvim\n""#,
    );
    write(&home.join(".zshrc"), "export EDITOR=vi\n");
    write_state(&home, &repo);

    luadot(&home).arg("diff").assert().success().stdout(
        predicate::str::contains("1 template(s) skipped")
            .and(predicate::str::contains(".zshrc").not()),
    );

    luadot(&home)
        .args(["diff", "--templates"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("diff --git a/home/.zshrc b/home/.zshrc")
                .and(predicate::str::contains("-export EDITOR=nvim"))
                .and(predicate::str::contains("+export EDITOR=vi"))
                .and(predicate::str::contains("1 of 1 generated file(s) differ")),
        );

    assert_eq!(read(&home.join(".zshrc")), "export EDITOR=vi\n");
}

fn mirrors(temp: &Path) -> usize {
    std::fs::read_dir(temp)
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

const FAKE_AGE: &str = r#"#!/bin/sh
[ -n "$FAKE_AGE_LOG" ] && printf '%s\n' "$*" >> "$FAKE_AGE_LOG"
op= out= src=
while [ $# -gt 0 ]; do
  case "$1" in
    -e|--encrypt) op=encrypt ;;
    -d|--decrypt) op=decrypt ;;
    -p|--passphrase) ;;
    -r|--recipient|-i|--identity) shift ;;
    -o|--output) out="$2"; shift ;;
    *) src="$1" ;;
  esac
  shift
done
if [ "$op" = encrypt ]; then
  {
    echo FAKEAGE
    if [ -n "$src" ]; then base64 "$src"; else base64; fi
  } > "${out:-/dev/stdout}"
elif [ "$op" = decrypt ]; then
  [ -n "$src" ] || { echo "fake age: no input" >&2; exit 1; }
  head -n 1 "$src" | grep -qx FAKEAGE || { echo "fake age: not a ciphertext" >&2; exit 1; }
  tail -n +2 "$src" | base64 -d > "${out:-/dev/stdout}"
else
  echo "fake age: no mode" >&2
  exit 1
fi
"#;

fn executable(path: &Path, script: &str) {
    use std::os::unix::fs::PermissionsExt;

    write(path, script);
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).unwrap();
}

fn fake_age(root: &Path) -> PathBuf {
    let bin = root.join("bin");
    std::fs::create_dir_all(&bin).unwrap();
    executable(&bin.join("age"), FAKE_AGE);
    bin
}

fn luadot_with_tools(home: &Path, bin: &Path) -> Command {
    let mut command = luadot(home);
    command.env(
        "PATH",
        format!("{}:{}", bin.display(), std::env::var("PATH").unwrap()),
    );
    command
}

fn crypt_config(home: &Path) {
    write(
        &home.join(".config/luadot/config.lua"),
        r#"
        ld.crypt.lock({ recipients = "age1example", identity = "~/key.txt" })
        ld.rules({ match = "home/.netrc", encrypt = true })
        "#,
    );
    write(&home.join("key.txt"), "AGE-SECRET-KEY-FAKE\n");
}

#[test]
fn an_encrypt_rule_keeps_only_ciphertext_in_the_repository() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    let repo = root.path().join("repo");
    std::fs::create_dir_all(&repo).unwrap();
    let bin = fake_age(root.path());
    crypt_config(&home);
    write(&home.join(".netrc"), "machine example password hunter2\n");
    write_state(&home, &repo);

    luadot_with_tools(&home, &bin)
        .args(["add", home.join(".netrc").to_str().unwrap()])
        .assert()
        .success();

    let cipher = read(&repo.join("home/.netrc.age"));
    assert!(cipher.starts_with("FAKEAGE\n"));
    assert!(!cipher.contains("hunter2"));
    assert!(!repo.join("home/.netrc").exists());

    std::fs::remove_file(home.join(".netrc")).unwrap();
    luadot_with_tools(&home, &bin)
        .arg("apply")
        .assert()
        .success();
    assert_eq!(
        read(&home.join(".netrc")),
        "machine example password hunter2\n"
    );

    use std::os::unix::fs::PermissionsExt;
    let mode = std::fs::metadata(home.join(".netrc"))
        .unwrap()
        .permissions()
        .mode()
        & 0o7777;
    assert_eq!(mode, 0o600);

    luadot_with_tools(&home, &bin)
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "nothing to apply, every managed file is synced",
        ));
}

#[test]
fn edit_reencrypts_and_leaves_no_plaintext_behind() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    let repo = root.path().join("repo");
    std::fs::create_dir_all(&repo).unwrap();
    let bin = fake_age(root.path());
    executable(
        &bin.join("append-editor"),
        "#!/bin/sh\nprintf 'password hunter3\\n' >> \"$1\"\n",
    );
    crypt_config(&home);
    write(&home.join(".netrc"), "machine example password hunter2\n");
    write_state(&home, &repo);

    luadot_with_tools(&home, &bin)
        .args(["add", home.join(".netrc").to_str().unwrap()])
        .assert()
        .success();

    luadot_with_tools(&home, &bin)
        .env("EDITOR", "append-editor")
        .args(["edit", home.join(".netrc").to_str().unwrap()])
        .assert()
        .success();

    let entries: Vec<String> = std::fs::read_dir(repo.join("home"))
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(entries, [".netrc.age"]);
    assert!(read(&repo.join("home/.netrc.age")).starts_with("FAKEAGE\n"));

    std::fs::remove_file(home.join(".netrc")).unwrap();
    luadot_with_tools(&home, &bin)
        .arg("apply")
        .assert()
        .success();
    assert_eq!(
        read(&home.join(".netrc")),
        "machine example password hunter2\npassword hunter3\n"
    );
}

#[test]
fn decrypting_with_age_asks_for_an_identity() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    let repo = root.path().join("repo");
    std::fs::create_dir_all(&repo).unwrap();
    let bin = fake_age(root.path());
    write(
        &home.join(".config/luadot/config.lua"),
        r#"
        ld.crypt.lock({ recipients = "age1example" })
        ld.rules({ match = "home/.netrc", encrypt = true })
        "#,
    );
    write(&home.join(".netrc"), "machine example password hunter2\n");
    write_state(&home, &repo);

    luadot_with_tools(&home, &bin)
        .args(["add", home.join(".netrc").to_str().unwrap()])
        .assert()
        .success();

    std::fs::remove_file(home.join(".netrc")).unwrap();
    luadot_with_tools(&home, &bin)
        .arg("apply")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "decrypting with age needs `ld.crypt.lock` with `identity`",
        ));
}

#[test]
fn rm_restores_the_plaintext_of_an_encrypted_file() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    let repo = root.path().join("repo");
    std::fs::create_dir_all(&repo).unwrap();
    let bin = fake_age(root.path());
    crypt_config(&home);
    write(&home.join(".netrc"), "machine example password hunter2\n");
    write_state(&home, &repo);

    luadot_with_tools(&home, &bin)
        .args(["add", home.join(".netrc").to_str().unwrap()])
        .assert()
        .success();
    std::fs::remove_file(home.join(".netrc")).unwrap();

    luadot_with_tools(&home, &bin)
        .args(["rm", "--yes", home.join(".netrc").to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("1 restored"));

    assert!(!repo.join("home/.netrc.age").exists());
    assert_eq!(
        read(&home.join(".netrc")),
        "machine example password hunter2\n"
    );
}

fn system_secret_config(home: &Path, rule: &str) {
    write(
        &home.join(".config/luadot/config.lua"),
        &format!(
            r#"
        ld.crypt.lock({{ recipients = "age1example", identity = "~/key.txt" }})
        ld.rules({{ match = "root/**", encrypt = true, {rule} }})
        "#
        ),
    );
    write(&home.join("key.txt"), "AGE-SECRET-KEY-FAKE\n");
}

fn stored_beside(system: &Path, repo: &Path) -> PathBuf {
    repo.join("root").join(format!(
        "{}.age",
        system.strip_prefix("/").unwrap().display()
    ))
}

#[test]
fn an_encrypt_rule_reaches_a_system_file() {
    use std::os::unix::fs::PermissionsExt;

    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    let repo = root.path().join("repo");
    std::fs::create_dir_all(&repo).unwrap();
    let bin = fake_age(root.path());
    let system = root.path().join("etc/wireguard/wg0.conf");
    write(&system, "PrivateKey = hunter2\n");
    system_secret_config(&home, "");
    write_state(&home, &repo);

    luadot_with_tools(&home, &bin)
        .args(["add", system.to_str().unwrap()])
        .assert()
        .success();

    let cipher = read(&stored_beside(&system, &repo));
    assert!(cipher.starts_with("FAKEAGE\n"));
    assert!(!cipher.contains("hunter2"));

    std::fs::remove_file(&system).unwrap();
    luadot_with_tools(&home, &bin)
        .arg("apply")
        .assert()
        .success();

    assert_eq!(read(&system), "PrivateKey = hunter2\n");
    assert_eq!(
        std::fs::metadata(&system).unwrap().permissions().mode() & 0o7777,
        0o600
    );

    luadot_with_tools(&home, &bin)
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "nothing to apply, every managed file is synced",
        ));
}

#[test]
fn rekey_re_encrypts_every_secret_for_the_recipients_set_now() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    let repo = root.path().join("repo");
    std::fs::create_dir_all(&repo).unwrap();
    let bin = fake_age(root.path());
    crypt_config(&home);
    write(&home.join(".netrc"), "machine example password hunter2\n");
    write_state(&home, &repo);

    luadot_with_tools(&home, &bin)
        .args(["add", home.join(".netrc").to_str().unwrap()])
        .assert()
        .success();

    write(
        &home.join(".config/luadot/config.lua"),
        r#"
        ld.crypt.lock({ recipients = { "age1example", "age1second" }, identity = "~/key.txt" })
        ld.rules({ match = "home/.netrc", encrypt = true })
        "#,
    );

    let log = root.path().join("age.log");
    luadot_with_tools(&home, &bin)
        .env("FAKE_AGE_LOG", &log)
        .arg("rekey")
        .assert()
        .success()
        .stdout(predicate::str::contains("re-encrypted 1 secret(s)"));

    let calls = read(&log);
    assert!(calls.contains("--recipient age1second"), "{calls}");
    assert!(!read(&repo.join("home/.netrc.age")).contains("hunter2"));

    std::fs::remove_file(home.join(".netrc")).unwrap();
    luadot_with_tools(&home, &bin)
        .arg("apply")
        .assert()
        .success();
    assert_eq!(
        read(&home.join(".netrc")),
        "machine example password hunter2\n"
    );
}

#[test]
fn rekey_reports_what_it_would_do_without_touching_the_repository() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    let repo = root.path().join("repo");
    std::fs::create_dir_all(&repo).unwrap();
    let bin = fake_age(root.path());
    crypt_config(&home);
    write(&home.join(".netrc"), "machine example password hunter2\n");
    write_state(&home, &repo);

    luadot_with_tools(&home, &bin)
        .args(["add", home.join(".netrc").to_str().unwrap()])
        .assert()
        .success();
    let before = read(&repo.join("home/.netrc.age"));

    luadot_with_tools(&home, &bin)
        .args(["rekey", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("would re-encrypt 1 secret(s)"));

    assert_eq!(read(&repo.join("home/.netrc.age")), before);
}

#[test]
fn passphrase_mode_says_it_is_weaker_and_the_warning_can_be_silenced() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    let repo = root.path().join("repo");
    std::fs::create_dir_all(&repo).unwrap();
    let bin = fake_age(root.path());
    write(
        &home.join(".config/luadot/config.lua"),
        r#"
        ld.crypt.lock("passphrase")
        ld.rules({ match = "home/.netrc", encrypt = true })
        "#,
    );
    write(&home.join(".netrc"), "machine example password hunter2\n");
    write_state(&home, &repo);

    let log = root.path().join("age.log");
    luadot_with_tools(&home, &bin)
        .env("FAKE_AGE_LOG", &log)
        .args(["add", home.join(".netrc").to_str().unwrap()])
        .assert()
        .success()
        .stderr(predicate::str::contains(
            "passphrase mode is weaker than keys",
        ));

    let calls = read(&log);
    assert!(calls.contains("--passphrase"), "{calls}");
    assert!(!calls.contains("--recipient"), "{calls}");
    assert!(!read(&repo.join("home/.netrc.age")).contains("hunter2"));

    std::fs::remove_file(home.join(".netrc")).unwrap();
    luadot_with_tools(&home, &bin)
        .arg("apply")
        .assert()
        .success();
    assert_eq!(
        read(&home.join(".netrc")),
        "machine example password hunter2\n"
    );

    write(
        &home.join(".config/luadot/config.lua"),
        r#"
        ld.crypt.lock("passphrase")
        ld.opt.passphrase_warn(false)
        ld.rules({ match = "home/.netrc", encrypt = true })
        "#,
    );
    luadot_with_tools(&home, &bin)
        .arg("status")
        .assert()
        .success()
        .stderr(predicate::str::contains("passphrase mode").not());
}

#[test]
fn an_identity_command_hands_the_key_over_without_a_file() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    let repo = root.path().join("repo");
    std::fs::create_dir_all(&repo).unwrap();
    let bin = fake_age(root.path());
    write(
        &home.join(".config/luadot/config.lua"),
        r#"
        ld.crypt.lock({ recipients = "age1example", identity = "printf 'AGE-SECRET-KEY-FAKE\n'" })
        ld.rules({ match = "home/.netrc", encrypt = true })
        "#,
    );
    write(&home.join(".netrc"), "machine example password hunter2\n");
    write_state(&home, &repo);

    luadot_with_tools(&home, &bin)
        .args(["add", home.join(".netrc").to_str().unwrap()])
        .assert()
        .success();
    std::fs::remove_file(home.join(".netrc")).unwrap();

    let log = root.path().join("age.log");
    luadot_with_tools(&home, &bin)
        .env("FAKE_AGE_LOG", &log)
        .arg("apply")
        .assert()
        .success();

    assert_eq!(
        read(&home.join(".netrc")),
        "machine example password hunter2\n"
    );
    assert!(read(&log).contains("--identity"), "{}", read(&log));
}

#[test]
fn a_failing_identity_command_stops_the_command() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    let repo = root.path().join("repo");
    std::fs::create_dir_all(&repo).unwrap();
    let bin = fake_age(root.path());
    crypt_config(&home);
    write(&home.join(".netrc"), "machine example password hunter2\n");
    write_state(&home, &repo);

    luadot_with_tools(&home, &bin)
        .args(["add", home.join(".netrc").to_str().unwrap()])
        .assert()
        .success();
    std::fs::remove_file(home.join(".netrc")).unwrap();

    write(
        &home.join(".config/luadot/config.lua"),
        r#"
        ld.crypt.lock({ recipients = "age1example", identity = { type = "command", "echo locked >&2; exit 1" } })
        ld.rules({ match = "home/.netrc", encrypt = true })
        "#,
    );

    luadot_with_tools(&home, &bin)
        .arg("apply")
        .assert()
        .failure()
        .stderr(predicate::str::contains("locked"));
}

#[test]
fn config_repo_prints_the_managed_repository() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    let repo = root.path().join("repo");
    std::fs::create_dir_all(&repo).unwrap();
    write_state(&home, &repo);

    luadot(&home)
        .args(["config", "repo"])
        .assert()
        .success()
        .stdout(predicate::str::contains(repo.to_str().unwrap()));
}

#[test]
fn setup_lists_the_names_the_repository_declares() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    let repo = root.path().join("repo");
    write(&repo.join("home/.config/luadot/setup/packages.lua"), "");
    write(&repo.join("home/.config/luadot/setup/shell.sh"), "");
    write_state(&home, &repo);

    luadot(&home)
        .args(["setup", "--list"])
        .assert()
        .success()
        .stdout(predicate::str::diff("packages\nshell\n"));
}
