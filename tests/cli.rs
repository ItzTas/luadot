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

fn lfs_available() -> bool {
    std::process::Command::new("git-lfs")
        .arg("version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

fn staged(repo: &Path) -> String {
    git(repo, &["diff", "--cached", "--name-only"])
}

fn subject(repo: &Path) -> String {
    git(repo, &["log", "-1", "--format=%s"])
}

fn backed(saved: &Path, path: &Path) -> PathBuf {
    saved.join(path.strip_prefix("/").unwrap())
}

fn write_state(home: &Path, repo: &Path) {
    write(
        &home.join(".local/share/luadot/state.json"),
        &format!(r#"{{"repo":{:?},"classes":{{}}}}"#, repo.display()),
    );
}

#[test]
fn exec_runs_inline_lua() {
    let home = tempfile::tempdir().unwrap();

    luadot(home.path())
        .args(["exec", r#"print("hi from lua")"#])
        .assert()
        .success()
        .stdout(predicate::str::contains("hi from lua"));
}

#[test]
fn add_then_apply_end_to_end() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    let repo = root.path().join("repo");
    std::fs::create_dir_all(&repo).unwrap();
    write(&home.join(".vimrc"), "set number\n");
    write_state(&home, &repo);

    luadot(&home)
        .args(["add", home.join(".vimrc").to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("added      .vimrc"))
        .stdout(predicate::str::contains("added 1 file(s)"));
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
        .stdout(predicate::str::contains(
            "nothing to apply, every managed file is synced",
        ));
}

#[test]
fn symbolic_rule_links_system_copy() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    let repo = root.path().join("repo");
    repository(&repo);
    write(&home.join(".config/tlp/tlp.conf"), "TLP_ENABLE=1\n");
    write(
        &home.join(".config/luadot/config.lua"),
        r#"ld.rules({ { match = ".config/tlp/**", link = "symbolic" } })"#,
    );
    write_state(&home, &repo);

    luadot_with_git(&home)
        .args(["add", home.join(".config/tlp").to_str().unwrap()])
        .assert()
        .success();

    let stored = repo.join(".config/tlp/tlp.conf");
    let placed = home.join(".config/tlp/tlp.conf");
    assert!(stored.symlink_metadata().unwrap().file_type().is_file());
    assert_eq!(read(&stored), "TLP_ENABLE=1\n");
    assert!(git(&repo, &["ls-files", "-s"]).starts_with("100644 "));
    assert_eq!(std::fs::read_link(&placed).unwrap(), stored);

    luadot_with_git(&home)
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "nothing to apply, every managed file is synced",
        ));
}

#[test]
fn whole_rule_links_the_directory_itself() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    let repo = root.path().join("repo");
    repository(&repo);
    write(&home.join(".config/nvim/init.lua"), "init\n");
    write(&home.join(".config/nvim/lua/plugins.lua"), "plugins\n");
    write(
        &home.join(".config/luadot/config.lua"),
        r#"ld.rules({ match = ".config/nvim", whole = true, link = "symbolic" })"#,
    );
    write_state(&home, &repo);

    luadot_with_git(&home)
        .args(["add", home.join(".config/nvim").to_str().unwrap()])
        .assert()
        .success();

    let stored = repo.join(".config/nvim");
    let placed = home.join(".config/nvim");
    assert_eq!(std::fs::read_link(&placed).unwrap(), stored);
    assert_eq!(read(&stored.join("lua/plugins.lua")), "plugins\n");

    write(&placed.join("keymaps.lua"), "maps\n");
    assert_eq!(read(&stored.join("keymaps.lua")), "maps\n");

    luadot_with_git(&home)
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "nothing to apply, every managed file is synced",
        ));

    luadot_with_git(&home)
        .args(["rm", "-y", placed.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("restored"));

    assert!(!placed.symlink_metadata().unwrap().file_type().is_symlink());
    assert_eq!(read(&placed.join("init.lua")), "init\n");
    assert_eq!(read(&placed.join("keymaps.lua")), "maps\n");
    assert!(!stored.exists());
}

#[test]
fn apply_places_a_whole_directory_as_one_link() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    let repo = root.path().join("repo");
    std::fs::create_dir_all(&repo).unwrap();
    write(&repo.join(".config/nvim/init.lua"), "init\n");
    write(
        &home.join(".config/luadot/config.lua"),
        r#"ld.rules({ match = ".config/nvim", whole = true, link = "symbolic" })"#,
    );
    write_state(&home, &repo);

    luadot(&home).arg("apply").assert().success();

    let placed = home.join(".config/nvim");
    assert_eq!(
        std::fs::read_link(&placed).unwrap(),
        repo.join(".config/nvim")
    );

    luadot(&home)
        .arg("apply")
        .assert()
        .success()
        .stdout(predicate::str::contains("1 unchanged"));
}

#[test]
fn add_and_rm_stage_their_changes() {
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
    assert_eq!(staged(&repo), ".vimrc\n");

    git(&repo, &["commit", "--quiet", "-m", "first"]);
    luadot_with_git(&home)
        .args(["rm", "-y", home.join(".vimrc").to_str().unwrap()])
        .assert()
        .success();

    assert_eq!(staged(&repo), ".vimrc\n");
    assert_eq!(read(&home.join(".vimrc")), "set number\n");
}

#[test]
fn take_stores_system_copy_and_relinks() {
    use std::os::unix::fs::MetadataExt;

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
    git(&repo, &["commit", "--quiet", "-m", "first"]);

    std::fs::remove_file(home.join(".vimrc")).unwrap();
    write(&home.join(".vimrc"), "set paste\n");

    luadot_with_git(&home)
        .args(["take", home.join(".vimrc").to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "took 1 file(s) (0 added, 1 replaced)",
        ));

    assert_eq!(read(&repo.join(".vimrc")), "set paste\n");
    assert_eq!(staged(&repo), ".vimrc\n");

    let (system, stored) = (
        std::fs::metadata(home.join(".vimrc")).unwrap(),
        std::fs::metadata(repo.join(".vimrc")).unwrap(),
    );
    assert_eq!((system.dev(), system.ino()), (stored.dev(), stored.ino()));
}

#[test]
fn take_with_no_path_stores_everything_and_backs_it_up() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    let repo = root.path().join("repo");
    repository(&repo);
    write(&repo.join(".vimrc"), "set number\n");
    write(&repo.join(".bashrc"), "managed\n");
    write(&home.join(".vimrc"), "set paste\n");
    write(
        &home.join(".config/luadot/config.lua"),
        r#"ld.opt.backup_dir("~/saved")"#,
    );
    write_state(&home, &repo);

    luadot_with_git(&home)
        .arg("take")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "took 1 file(s) (0 added, 1 replaced)",
        ));

    assert_eq!(read(&repo.join(".vimrc")), "set paste\n");
    assert_eq!(read(&repo.join(".bashrc")), "managed\n");
    assert_eq!(staged(&repo), ".vimrc\n");

    let saved = only_dir(&home.join("saved"));
    assert_eq!(read(&backed(&saved, &repo.join(".vimrc"))), "set number\n");
}

#[test]
fn add_and_take_point_at_each_other() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    let repo = root.path().join("repo");
    repository(&repo);
    write(&home.join(".vimrc"), "set number\n");
    write_state(&home, &repo);

    luadot(&home)
        .args(["take", home.join(".vimrc").to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "is not in the repository; run `luadot add` to start managing it",
        ));

    luadot_with_git(&home)
        .args(["add", home.join(".vimrc").to_str().unwrap()])
        .assert()
        .success();

    luadot(&home)
        .args(["add", home.join(".vimrc").to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "already exists in the repository; run `luadot take` to store what the system holds",
        ));
}

#[test]
fn mv_moves_both_sides_and_stages() {
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
    git(&repo, &["commit", "--quiet", "-m", "first"]);

    luadot_with_git(&home)
        .args([
            "mv",
            home.join(".vimrc").to_str().unwrap(),
            home.join(".config/vim/vimrc").to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "moved      .vimrc -> .config/vim/vimrc",
        ))
        .stdout(predicate::str::contains("moved 1 file(s)"));

    assert!(!repo.join(".vimrc").exists());
    assert!(!home.join(".vimrc").exists());
    assert_eq!(read(&repo.join(".config/vim/vimrc")), "set number\n");
    assert_eq!(read(&home.join(".config/vim/vimrc")), "set number\n");
    assert_eq!(
        git(&repo, &["diff", "--cached", "--name-status"]),
        "R100\t.vimrc\t.config/vim/vimrc\n"
    );
}

#[test]
fn mv_dry_run_moves_nothing() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    let repo = root.path().join("repo");
    write(&repo.join(".vimrc"), "set number\n");
    write(&home.join(".vimrc"), "set number\n");
    write_state(&home, &repo);

    luadot(&home)
        .args([
            "mv",
            "-n",
            home.join(".vimrc").to_str().unwrap(),
            home.join(".gvimrc").to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("would move 1 file(s)"));

    assert!(repo.join(".vimrc").exists());
    assert!(home.join(".vimrc").exists());
    assert!(!repo.join(".gvimrc").exists());
}

#[test]
fn add_stores_in_lfs() {
    if !lfs_available() {
        return;
    }
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    let repo = root.path().join("repo");
    repository(&repo);
    write(
        &home.join(".config/luadot/config.lua"),
        r#"ld.rules({ match = "Videos/**", lfs = true })"#,
    );
    write(&home.join("Videos/clip.mp4"), "a clip\n");
    write_state(&home, &repo);

    luadot_with_git(&home)
        .args(["add", home.join("Videos/clip.mp4").to_str().unwrap()])
        .assert()
        .success();

    assert_eq!(
        staged(&repo),
        ".local/share/luadot/git/attributes\nVideos/clip.mp4\n"
    );
    assert!(
        read(&repo.join(".local/share/luadot/git/attributes"))
            .contains("Videos/** filter=lfs diff=lfs merge=lfs")
    );
    assert!(
        read(&repo.join(".git/info/attributes"))
            .contains("Videos/** filter=lfs diff=lfs merge=lfs")
    );
    assert!(
        git(&repo, &["cat-file", "-p", ":Videos/clip.mp4"])
            .starts_with("version https://git-lfs.github.com/spec/v1")
    );
}

#[test]
fn rules_reach_git_info_files() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    let repo = root.path().join("repo");
    repository(&repo);
    write(&repo.join(".local/share/luadot/git/ignore"), "*.log\n");
    write(&repo.join("noise.log"), "x\n");
    write(&repo.join(".vimrc"), "set number\n");
    write_state(&home, &repo);

    luadot_with_git(&home)
        .args(["git", "status", "--porcelain", "--untracked-files=all"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains(".vimrc").and(predicate::str::contains("noise.log").not()),
        );

    assert!(read(&repo.join(".git/info/exclude")).ends_with("# luadot\n*.log\n# /luadot\n"));
    assert!(!git(&repo, &["status", "--porcelain", "--untracked-files=all"]).contains("noise.log"));
}

#[test]
fn sync_commits_then_finds_nothing() {
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
fn class_asked_for_then_read_from_state() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    let repo = root.path().join("repo");
    write(
        &repo.join(".zshrc.luadot/luadot.lua"),
        r#"
        ld.class({ name = "editor", choices = { "nvim", "helix" } })
        return "EDITOR=" .. ld.class.get("editor") .. "\n"
        "#,
    );
    write_state(&home, &repo);

    luadot(&home)
        .args(["status", "--templates"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("luadot class set editor"));

    write(
        &home.join(".local/share/luadot/state.json"),
        &format!(
            r#"{{"repo":{:?},"classes":{{"editor":"nvim"}}}}"#,
            repo.display()
        ),
    );

    luadot(&home)
        .args(["status", "--templates"])
        .assert()
        .success()
        .stdout(predicate::str::contains(".zshrc"));
}

#[test]
fn setup_overrides_the_configuration() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    let repo = root.path().join("repo");
    write(&repo.join(".bashrc"), "managed\n");
    write(
        &repo.join(".config/luadot/setup/dots.lua"),
        r#"ld.opt.link("symbolic")"#,
    );
    write(
        &home.join(".config/luadot/config.lua"),
        r#"
        ld.opt.link("hard")
        ld.setup("dots")
        "#,
    );
    write_state(&home, &repo);

    luadot(&home).arg("apply").assert().success();

    assert!(
        std::fs::symlink_metadata(home.join(".bashrc"))
            .unwrap()
            .file_type()
            .is_symlink()
    );
}

#[test]
fn apply_backs_up_where_restore_finds_it() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    let repo = root.path().join("repo");
    write(&repo.join(".bashrc"), "managed\n");
    write(&home.join(".bashrc"), "handwritten\n");
    write(
        &home.join(".config/luadot/config.lua"),
        r#"ld.opt.backup_dir("~/saved")"#,
    );
    write_state(&home, &repo);

    luadot(&home).arg("apply").assert().success();

    let saved = only_dir(&home.join("saved"));
    assert_eq!(
        read(&backed(&saved, &home.join(".bashrc"))),
        "handwritten\n"
    );
    assert_eq!(read(&home.join(".bashrc")), "managed\n");
    assert!(!home.join(".local/share/luadot/backups").exists());

    let stamp = saved.file_name().unwrap().to_str().unwrap().to_string();
    let dest = home.join(".bashrc").display().to_string();

    luadot(&home)
        .args(["restore", "--list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("1 file(s)"));

    luadot(&home)
        .args(["restore", "--list", &stamp])
        .assert()
        .success()
        .stdout(predicate::str::contains(dest.clone()));

    luadot(&home)
        .args(["restore", "--yes"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains(dest).and(predicate::str::contains("(0 created, 1 replaced)")),
        );
    assert_eq!(read(&home.join(".bashrc")), "handwritten\n");
}

#[test]
fn init_creates_and_adopts_repository() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    let repo = root.path().join("dotfiles");
    write(&home.join(".vimrc"), "set number\n");

    luadot(&home)
        .args(["init", repo.to_str().unwrap()])
        .assert()
        .success();
    assert!(repo.join(".git").is_dir());
    assert!(
        read(&home.join(".config/luadot/config.lua")).starts_with("-- The luadot configuration")
    );

    luadot(&home)
        .args(["add", home.join(".vimrc").to_str().unwrap()])
        .assert()
        .success();
    assert_eq!(read(&repo.join(".vimrc")), "set number\n");
}

#[test]
fn clone_keeps_repository_settings() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    let source = root.path().join("source");
    let repo = root.path().join("dotfiles");
    let settings = "{\n  \"diagnostics.globals\": [\"vim\"]\n}\n";
    repository(&source);
    write(&source.join(".luarc.json"), settings);
    git(&source, &["add", ".luarc.json"]);
    git(&source, &["commit", "--quiet", "-m", "settings"]);

    luadot_with_git(&home)
        .args(["clone", source.to_str().unwrap(), repo.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("wrote"));

    assert_eq!(read(&repo.join(".luarc.json")), settings);
    assert!(
        read(&home.join(".config/luadot/.luarc.json")).contains("\"~/.local/share/luadot/meta\"")
    );
    assert!(home.join(".local/share/luadot/meta/ld.lua").is_file());
}

#[test]
fn alt_resolves_forms_apply_skips_them() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    let repo = root.path().join("repo");
    write(
        &repo.join(".zshrc.luadot/luadot.lua"),
        r#"return ld.alt.expand("zshrc.tmpl.zsh", { editor = "nvim" })"#,
    );
    write(
        &repo.join(".zshrc.luadot/zshrc.tmpl.zsh"),
        "export EDITOR=<%= editor %>\n",
    );
    write(
        &repo.join(".zprofile.luadot"),
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
fn tmpl_new_creates_both_forms() {
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
            "added .config/nvim/init.lua.luadot",
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
        read(&repo.join(".config/nvim/init.lua.luadot/luadot.lua")),
        "return \"\"\n"
    );
    assert_eq!(read(&home.join(".zprofile.luadot")), "");
    assert_eq!(read(&repo.join(".zprofile.luadot")), "");

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
fn rule_runs_only_on_touched_files() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    let repo = root.path().join("repo");
    let reloaded = root.path().join("reloaded");
    write(&repo.join(".config/mako/config"), "font=monospace\n");
    write(&repo.join(".config/mako/colors"), "background=#000000\n");
    write(&repo.join(".bashrc"), "managed\n");
    write(
        &home.join(".config/luadot/config.lua"),
        &format!(
            r#"ld.rules({{ {{ match = ".config/mako/**", on_change = "printf x >> {}" }} }})"#,
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
fn mode_rule_is_reapplied_on_drift() {
    use std::os::unix::fs::PermissionsExt;

    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    let repo = root.path().join("repo");
    let placed = home.join(".ssh/config");
    write(&repo.join(".ssh/config"), "Host example\n");
    write(
        &home.join(".config/luadot/config.lua"),
        r#"ld.rules({ match = ".ssh/**", link = "copy", mode = "0600" })"#,
    );
    write_state(&home, &repo);

    luadot(&home).arg("apply").assert().success();

    assert_eq!(read(&placed), "Host example\n");
    assert_eq!(
        std::fs::metadata(&placed).unwrap().permissions().mode() & 0o7777,
        0o600
    );

    luadot(&home)
        .arg("apply")
        .assert()
        .success()
        .stdout(predicate::str::contains("1 unchanged"));

    std::fs::set_permissions(&placed, std::fs::Permissions::from_mode(0o644)).unwrap();
    luadot(&home)
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("        differs:     .ssh/config"));
    luadot(&home)
        .arg("diff")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "mode       .ssh/config 0644 -> 0600",
        ));

    luadot(&home)
        .arg("apply")
        .assert()
        .success()
        .stdout(predicate::str::contains("1 replaced"));
    assert_eq!(
        std::fs::metadata(&placed).unwrap().permissions().mode() & 0o7777,
        0o600
    );
}

#[test]
fn diff_reports_drift_and_missing() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    let repo = root.path().join("repo");
    write(&repo.join(".bashrc"), "managed\n");
    write(&repo.join(".vimrc"), "set number\n");
    write(&repo.join(".zshrc"), "synced\n");
    write(&home.join(".bashrc"), "handwritten\n");
    write(&home.join(".zshrc"), "synced\n");
    write_state(&home, &repo);

    luadot(&home).arg("diff").assert().success().stdout(
        predicate::str::contains("diff --git a/.bashrc b/.bashrc")
            .and(predicate::str::contains("--- a/.bashrc"))
            .and(predicate::str::contains("+++ b/.bashrc"))
            .and(predicate::str::contains("-managed"))
            .and(predicate::str::contains("+handwritten"))
            .and(predicate::str::contains("diff --git a/.vimrc b/.vimrc"))
            .and(predicate::str::contains("deleted file mode 100644"))
            .and(predicate::str::contains("-set number"))
            .and(predicate::str::contains(".zshrc").not())
            .and(predicate::str::contains("2 of 3 managed file(s) differ")),
    );
}

#[test]
fn status_groups_files_by_state() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    let repo = root.path().join("repo");
    write(&repo.join(".bashrc"), "managed\n");
    write(&repo.join(".vimrc"), "set number\n");
    write(&home.join(".vimrc"), "set number\n");
    write(&repo.join(".zshrc"), "repository\n");
    write(&home.join(".zshrc"), "system\n");
    write_state(&home, &repo);

    luadot(&home).arg("status").assert().success().stdout(
        predicate::str::contains(format!("On repository {}", repo.display()))
            .and(predicate::str::contains("3 managed file(s)"))
            .and(predicate::str::contains("Files not on the system:"))
            .and(predicate::str::contains(
                "(use \"luadot apply <path>...\" to write them)",
            ))
            .and(predicate::str::contains("        missing:     .bashrc"))
            .and(predicate::str::contains("Files not linked:"))
            .and(predicate::str::contains("        unlinked:    .vimrc"))
            .and(predicate::str::contains("Files that differ:"))
            .and(predicate::str::contains(
                "(use \"luadot diff <path>...\" to see what changed)",
            ))
            .and(predicate::str::contains(
                "(use \"luadot apply\" to keep the repository's copy, \"luadot take\" to keep the system's)",
            ))
            .and(predicate::str::contains("        differs:     .zshrc")),
    );
}

#[test]
fn command_runs_before_and_after_hooks() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    let repo = root.path().join("repo");
    write(&repo.join(".bashrc"), "managed\n");
    write(
        &home.join(".config/luadot/config.lua"),
        r#"
        ld.on.apply({
          before = function() return "before " .. ld.argv.name end,
          after = function() ld.print("after " .. ld.argv.name) end,
        })
        ld.on.apply({ before = function() return "and again" end })
        "#,
    );
    write_state(&home, &repo);

    luadot(&home).arg("apply").assert().success().stdout(
        predicate::str::is_match("(?s)^before apply\nand again\n.*\\.bashrc.*\nafter apply\n$")
            .unwrap(),
    );
}

#[test]
fn task_runs_under_its_own_name() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    write(
        &home.join(".config/luadot/config.lua"),
        r#"
        ld.task("plug", {
          about = "Manage plugins",
          run = function(argv) return "plug " .. table.concat(argv, " ") end,
        })
        "#,
    );

    luadot(&home)
        .args(["plug", "sync", "--all"])
        .assert()
        .success()
        .stdout(predicate::str::diff("plug sync --all\n"));
    luadot(&home)
        .args(["task", "plug", "sync"])
        .assert()
        .success()
        .stdout(predicate::str::diff("plug sync\n"));
    luadot(&home)
        .args(["task", "--list"])
        .assert()
        .success()
        .stdout(predicate::str::diff("plug\n"));
    luadot(&home).arg("stauts").assert().code(2).stderr(
        predicate::str::contains("unrecognized subcommand 'stauts'")
            .and(predicate::str::contains("'status'")),
    );
    luadot(&home)
        .arg("plugg")
        .assert()
        .code(2)
        .stderr(predicate::str::contains("'plug'"));
    luadot(&home)
        .args(["task", "plugg"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "task: `plugg` is not a task the configuration registers (registered: plug)",
        ));
}

#[test]
fn doc_reads_a_page_even_when_config_breaks() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    write(
        &home.join("plugins/lazyld/docs/lazyld.md"),
        "| Call | Arguments | Effect |\n| --- | --- | --- |\n\
         | `lazyld.sync(names)` | plugin names | Clones what is missing. |\n",
    );
    write(
        &home.join(".config/luadot/config.lua"),
        r#"ld.doc.page("plugins/lazyld/docs/lazyld.md")"#,
    );

    luadot(&home)
        .args(["doc", "lazyld.sync"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("lazyld.sync(names)")
                .and(predicate::str::contains("Clones what is missing."))
                .and(predicate::str::contains("plugins/lazyld/docs/lazyld.md")),
        );

    write(&home.join(".config/luadot/config.lua"), "ld.opt.link(");
    luadot(&home)
        .args(["doc", "opt.link"])
        .assert()
        .success()
        .stdout(predicate::str::contains("ld.opt.link(mode)"))
        .stderr(predicate::str::contains("config: failed to run"));
}

#[test]
fn plugin_is_required_documented_and_run() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    let plugin = home.join(".local/share/luadot/plugins/lazyld");
    write(
        &plugin.join("lua/lazyld.lua"),
        r#"
        local lazyld = {}

        function lazyld.setup()
          ld.task("plug", { run = function(argv) return "plug " .. table.concat(argv, " ") end })
        end

        function lazyld.greet()
          return "hello from lazyld"
        end

        return lazyld
        "#,
    );
    write(
        &plugin.join("docs/lazyld.md"),
        "| Call | Arguments | Effect |\n| --- | --- | --- |\n\
         | `lazyld.greet()` | none | Says hello. |\n",
    );
    write(
        &home.join(".config/luadot/config.lua"),
        r#"
        local dir = ld.path.data .. "/plugins/lazyld"
        ld.rtp.add(dir)
        ld.doc.page(dir .. "/docs/lazyld.md")
        require("lazyld").setup()
        "#,
    );

    luadot(&home)
        .args(["exec", r#"print(require("lazyld").greet())"#])
        .assert()
        .success()
        .stdout(predicate::str::diff("hello from lazyld\n"));
    luadot(&home)
        .args(["plug", "sync"])
        .assert()
        .success()
        .stdout(predicate::str::diff("plug sync\n"));
    luadot(&home)
        .args(["doc", "lazyld.greet"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Says hello."));
}

#[test]
fn print_writes_each_shape() {
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
fn rm_template_keeps_generated_file() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    let repo = root.path().join("repo");
    write(&repo.join(".zshrc.luadot/laptop.zsh"), "laptop\n");
    write(
        &repo.join(".zshrc.luadot/luadot.lua"),
        r#"return { content = ld.alt.file("laptop.zsh"), link = "symbolic" }"#,
    );
    write_state(&home, &repo);

    luadot(&home).args(["tmpl", "alt"]).assert().success();
    assert_eq!(
        std::fs::read_link(home.join(".zshrc")).unwrap(),
        repo.join(".zshrc.luadot/laptop.zsh")
    );

    luadot(&home)
        .args(["rm", "--dry-run", home.join(".zshrc").to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "would stop managing 1 template(s) (1 restored, 0 left untouched)",
        ));
    assert!(repo.join(".zshrc.luadot").is_dir());

    luadot(&home)
        .args(["rm", "--yes", home.join(".zshrc").to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "stopped managing 1 template(s) (1 restored, 0 left untouched)",
        ));

    assert!(!repo.join(".zshrc.luadot").exists());
    assert!(
        !std::fs::symlink_metadata(home.join(".zshrc"))
            .unwrap()
            .is_symlink()
    );
    assert_eq!(read(&home.join(".zshrc")), "laptop\n");
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
    if [ -n "$src" ]; then base64 < "$src"; else base64; fi
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
        ld.rules({ match = ".netrc", encrypt = true })
        "#,
    );
    write(&home.join("key.txt"), "AGE-SECRET-KEY-FAKE\n");
}

#[test]
fn every_secret_gets_its_own_plaintext() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    let repo = root.path().join("repo");
    std::fs::create_dir_all(&repo).unwrap();
    let bin = fake_age(root.path());
    write(
        &home.join(".config/luadot/config.lua"),
        r#"
        ld.crypt.lock({ recipients = "age1example", identity = "~/key.txt" })
        ld.rules({ match = ".secret*", encrypt = true })
        "#,
    );
    write(&home.join("key.txt"), "AGE-SECRET-KEY-FAKE\n");
    write_state(&home, &repo);

    let names: Vec<String> = (0..6).map(|index| format!(".secret{index}")).collect();
    for (index, name) in names.iter().enumerate() {
        write(&home.join(name), &format!("password hunter{index}\n"));
        luadot_with_tools(&home, &bin)
            .args(["add", home.join(name).to_str().unwrap()])
            .assert()
            .success();
        std::fs::remove_file(home.join(name)).unwrap();
    }

    let log = root.path().join("age.log");
    luadot_with_tools(&home, &bin)
        .env("FAKE_AGE_LOG", &log)
        .arg("apply")
        .assert()
        .success();

    let calls = read(&log);
    let decrypts = calls
        .lines()
        .filter(|line| line.contains("--decrypt"))
        .count();
    assert_eq!(decrypts, names.len(), "{calls}");

    for (index, name) in names.iter().enumerate() {
        assert_eq!(read(&home.join(name)), format!("password hunter{index}\n"));
    }
}

#[test]
fn diff_compares_the_plaintext_of_a_secret() {
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
    assert!(repo.join(".netrc.age").exists());

    write(&home.join(".netrc"), "machine example password changed\n");

    luadot_with_tools(&home, &bin)
        .arg("diff")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("a/.netrc")
                .and(predicate::str::contains(
                    "-machine example password hunter2",
                ))
                .and(predicate::str::contains(
                    "+machine example password changed",
                ))
                .and(predicate::str::contains("1 of 1 managed file(s) differ"))
                .and(predicate::str::contains("FAKEAGE").not())
                .and(predicate::str::contains(".netrc.age").not()),
        );

    luadot_with_tools(&home, &bin)
        .arg("status")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("differs:     .netrc")
                .and(predicate::str::contains(".netrc.age").not()),
        );
}

#[test]
fn take_re_encrypts_the_system_copy_of_a_secret() {
    use std::os::unix::fs::PermissionsExt;

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

    write(&home.join(".netrc"), "machine example password rotated\n");
    std::fs::set_permissions(home.join(".netrc"), std::fs::Permissions::from_mode(0o600)).unwrap();

    luadot_with_tools(&home, &bin)
        .args(["take", home.join(".netrc").to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "took 1 file(s) (0 added, 1 replaced)",
        ));

    let stored = read(&repo.join(".netrc.age"));
    assert!(stored.starts_with("FAKEAGE\n"), "{stored}");
    assert!(!stored.contains("rotated"), "{stored}");
    assert_eq!(std::fs::read_dir(&repo).unwrap().count(), 1);

    luadot_with_tools(&home, &bin)
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "nothing to apply, every managed file is synced",
        ));
}

#[test]
fn encrypt_rule_stores_only_ciphertext() {
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

    let cipher = read(&repo.join(".netrc.age"));
    assert!(cipher.starts_with("FAKEAGE\n"));
    assert!(!cipher.contains("hunter2"));
    assert!(!repo.join(".netrc").exists());

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
fn edit_leaves_no_plaintext() {
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

    let entries: Vec<String> = std::fs::read_dir(&repo)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(entries, [".netrc.age"]);
    assert!(read(&repo.join(".netrc.age")).starts_with("FAKEAGE\n"));

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
fn edit_hands_the_editor_a_private_plaintext() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    let repo = root.path().join("repo");
    let recorded = root.path().join("mode");
    std::fs::create_dir_all(&repo).unwrap();
    let bin = fake_age(root.path());
    executable(
        &bin.join("mode-editor"),
        &format!(
            "#!/bin/sh\nstat -c '%a' \"$1\" > {}\nprintf 'x\\n' >> \"$1\"\n",
            recorded.display()
        ),
    );
    crypt_config(&home);
    write(&home.join(".netrc"), "machine example password hunter2\n");
    write_state(&home, &repo);

    luadot_with_tools(&home, &bin)
        .args(["add", home.join(".netrc").to_str().unwrap()])
        .assert()
        .success();

    luadot_with_tools(&home, &bin)
        .env("EDITOR", "mode-editor")
        .args(["edit", home.join(".netrc").to_str().unwrap()])
        .assert()
        .success();

    assert_eq!(read(&recorded).trim(), "600");
}

#[test]
fn rekey_uses_the_new_recipients() {
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
        ld.rules({ match = ".netrc", encrypt = true })
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
    assert!(!read(&repo.join(".netrc.age")).contains("hunter2"));

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
fn identity_command_hands_the_key_over() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    let repo = root.path().join("repo");
    std::fs::create_dir_all(&repo).unwrap();
    let bin = fake_age(root.path());
    write(
        &home.join(".config/luadot/config.lua"),
        r#"
        ld.crypt.lock({ recipients = "age1example", identity = "printf 'AGE-SECRET-KEY-FAKE\n'" })
        ld.rules({ match = ".netrc", encrypt = true })
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
fn meta_install_merges_the_settings() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    let repo = root.path().join("repo");
    std::fs::create_dir_all(&repo).unwrap();
    write(
        &home.join(".config/luadot/.luarc.json"),
        "{\n  \"diagnostics.globals\": [\"vim\"]\n}\n",
    );
    write(
        &home.join(".config/luadot/config.lua"),
        r#"ld.rtp.add(".local/share/luadot/plugins/lazyld")"#,
    );
    write_state(&home, &repo);

    luadot(&home)
        .args(["meta", "install"])
        .assert()
        .success()
        .stdout(predicate::str::contains("merged"));

    let definitions = read(&home.join(".local/share/luadot/meta/ld.lua"));
    assert!(definitions.starts_with("---@meta\n"));
    assert!(!repo.join("meta/ld.lua").exists());
    assert!(!repo.join(".luarc.json").exists());

    let merged = read(&home.join(".config/luadot/.luarc.json"));
    assert!(merged.contains("\"diagnostics.globals\""));
    assert!(merged.contains("\"~/.local/share/luadot/meta\""));
    assert!(merged.contains("\"~/.local/share/luadot/plugins/lazyld\""));

    luadot(&home)
        .arg("meta")
        .assert()
        .success()
        .stdout(predicate::str::diff(definitions));
}

#[test]
fn doc_without_a_call_lists_every_call() {
    let home = tempfile::tempdir().unwrap();

    luadot(home.path())
        .arg("doc")
        .assert()
        .success()
        .stdout(predicate::str::contains("opt.link\n"))
        .stdout(predicate::str::contains(
            "\"luadot doc <call>\" to describe one",
        ));
}

#[test]
fn add_with_no_path_takes_what_an_auto_rule_covers() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    let repo = root.path().join("repo");
    repository(&repo);
    write(&home.join(".config/nvim/init.lua"), "vim.o.number = true\n");
    write(&home.join(".config/nvim/spell/en.add"), "luadot\n");
    write(&home.join(".bashrc"), "alias l=ls\n");
    write(
        &home.join(".config/luadot/config.lua"),
        r#"
        ld.rules({
          { match = ".config/nvim/**", track = "auto" },
          { match = ".config/nvim/spell/**", track = "never" },
        })
        "#,
    );
    write_state(&home, &repo);

    luadot_with_git(&home)
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("1 file(s) an `auto` rule covers"));

    luadot_with_git(&home)
        .arg("add")
        .assert()
        .success()
        .stdout(predicate::str::contains("added      .config/nvim/init.lua"))
        .stdout(predicate::str::contains("added 1 file(s)"));

    assert_eq!(
        read(&repo.join(".config/nvim/init.lua")),
        "vim.o.number = true\n"
    );
    assert!(!repo.join(".config/nvim/spell/en.add").exists());
    assert!(!repo.join(".bashrc").exists());
    assert_eq!(staged(&repo).trim(), ".config/nvim/init.lua");

    luadot_with_git(&home)
        .arg("add")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "the `auto` rules cover nothing new",
        ));
}
