use std::collections::BTreeMap;
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use glob::Pattern;
use mlua::Lua;
use regex::Regex;

use super::around::{Around, Chain};
use super::constants::{CLASS_QUESTION, GIT_DIR, LOCKED, MATCH, MISSING};
use super::diff::Diff;
use super::report::Report;
use super::task::Task;
use crate::backup::Retention;
use crate::crypt::{Backend, Identity, Lock, Secrets};
use crate::files::{ConflictPolicy, LinkMode, Placement};
use crate::lua::ld::Command;

pub type Shared = Arc<Mutex<Config>>;

#[derive(Debug, Clone)]
pub struct Config {
    link: LinkMode,
    conflict: ConflictPolicy,
    rules: Vec<Rule>,
    classes: Vec<Class>,
    pkg_warn: bool,
    passphrase_warn: bool,
    autocommit: bool,
    autopush: bool,
    lfs: bool,
    backup: bool,
    backup_dir: Option<PathBuf>,
    backup_keep: Option<u32>,
    backup_age: Option<u64>,
    repo_dir: Option<PathBuf>,
    crypt_backend: Backend,
    crypt_secrets: Secrets,
    diff: Diff,
    status: Report,
    around: BTreeMap<Command, Chain>,
    runtime_paths: Vec<PathBuf>,
    tasks: BTreeMap<String, Task>,
    runtimes: Vec<Lua>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            link: LinkMode::default(),
            conflict: ConflictPolicy::default(),
            rules: Vec::new(),
            classes: Vec::new(),
            pkg_warn: true,
            passphrase_warn: true,
            autocommit: false,
            autopush: false,
            lfs: true,
            backup: true,
            backup_dir: None,
            backup_keep: None,
            backup_age: None,
            repo_dir: None,
            crypt_backend: Backend::default(),
            crypt_secrets: Secrets::default(),
            diff: Diff::default(),
            status: Report::default(),
            around: BTreeMap::new(),
            runtime_paths: Vec::new(),
            tasks: BTreeMap::new(),
            runtimes: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Matcher {
    Glob(Pattern),
    Regex(Regex),
    Any(Vec<Matcher>),
}

#[derive(Debug, Clone)]
pub struct Rule {
    pattern: Matcher,
    link: Option<LinkMode>,
    conflict: Option<ConflictPolicy>,
    on_change: Option<String>,
    ignore: Option<bool>,
    mode: Option<u32>,
    owner: Option<String>,
    encrypt: Option<bool>,
    lfs: Option<bool>,
    autocommit: Option<bool>,
    autopush: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Class {
    name: String,
    prompt: Option<String>,
    choices: Vec<String>,
    default: Option<String>,
}

impl Config {
    pub fn shared(lua: &Lua) -> mlua::Result<Shared> {
        lua.app_data_ref::<Shared>()
            .map(|shared| Arc::clone(&shared))
            .ok_or_else(|| mlua::Error::external(MISSING))
    }

    pub fn building<T>(lua: &Lua, edit: impl FnOnce(&mut Config) -> T) -> mlua::Result<T> {
        let shared = Self::shared(lua)?;
        let mut config = shared
            .try_lock()
            .map_err(|_| mlua::Error::external(LOCKED))?;

        Ok(edit(&mut config))
    }

    pub fn keep_runtime(&mut self, runtime: Lua) {
        self.runtimes.push(runtime);
    }

    pub fn add_runtime_path(&mut self, dir: PathBuf) {
        if self.runtime_paths.contains(&dir) {
            return;
        }

        self.runtime_paths.push(dir);
    }

    pub fn runtime_paths(&self) -> &[PathBuf] {
        &self.runtime_paths
    }

    pub fn set_diff(&mut self, diff: Diff) {
        self.diff.merge(diff);
    }

    pub fn diff(&self) -> &Diff {
        &self.diff
    }

    pub fn set_status(&mut self, status: Report) {
        self.status.merge(status);
    }

    pub fn status(&self) -> &Report {
        &self.status
    }

    pub fn set_around(&mut self, command: Command, around: Around) {
        self.around.entry(command).or_default().add(around);
    }

    pub fn around(&self, command: Command) -> Option<&Chain> {
        self.around.get(&command)
    }

    pub fn set_link(&mut self, link: LinkMode) {
        self.link = link;
    }

    pub fn set_conflict(&mut self, conflict: ConflictPolicy) {
        self.conflict = conflict;
    }

    pub fn add_rules(&mut self, rules: Vec<Rule>) {
        self.rules.extend(rules);
    }

    pub fn add_task(&mut self, name: String, task: Task) -> bool {
        if self.tasks.contains_key(&name) {
            return false;
        }

        self.tasks.insert(name, task);
        true
    }

    pub fn task(&self, name: &str) -> Option<&Task> {
        self.tasks.get(name)
    }

    pub fn tasks(&self) -> impl Iterator<Item = (&str, &Task)> {
        self.tasks.iter().map(|(name, task)| (name.as_str(), task))
    }

    pub fn add_class(&mut self, class: Class) {
        self.classes.retain(|current| current.name != class.name);
        self.classes.push(class);
    }

    pub fn set_pkg_warn(&mut self, pkg_warn: bool) {
        self.pkg_warn = pkg_warn;
    }

    pub fn pkg_warn(&self) -> bool {
        self.pkg_warn
    }

    pub fn set_autocommit(&mut self, autocommit: bool) {
        self.autocommit = autocommit;
    }

    pub fn set_autopush(&mut self, autopush: bool) {
        self.autopush = autopush;
    }

    pub fn set_lfs(&mut self, lfs: bool) {
        self.lfs = lfs;
    }

    pub fn lfs(&self) -> bool {
        self.lfs
    }

    pub fn set_backup(&mut self, backup: bool) {
        self.backup = backup;
    }

    pub fn backup(&self) -> bool {
        self.backup
    }

    pub fn set_backup_dir(&mut self, dir: PathBuf) {
        self.backup_dir = Some(dir);
    }

    pub fn backup_dir(&self) -> Option<&Path> {
        self.backup_dir.as_deref()
    }

    pub fn set_backup_keep(&mut self, keep: u32) {
        self.backup_keep = Some(keep);
    }

    pub fn backup_keep(&self) -> Option<u32> {
        self.backup_keep
    }

    pub fn set_backup_age(&mut self, age: u64) {
        self.backup_age = Some(age);
    }

    pub fn backup_age(&self) -> Option<u64> {
        self.backup_age
    }

    pub fn retention(&self) -> Retention {
        Retention::new(self.backup_keep, self.backup_age)
    }

    pub fn set_repo_dir(&mut self, dir: PathBuf) {
        self.repo_dir = Some(dir);
    }

    pub fn repo_dir(&self) -> Option<&Path> {
        self.repo_dir.as_deref()
    }

    pub fn set_crypt_backend(&mut self, backend: Backend) {
        self.crypt_backend = backend;
    }

    pub fn crypt_backend(&self) -> Backend {
        self.crypt_backend
    }

    pub fn set_crypt_secrets(&mut self, secrets: Secrets) {
        self.crypt_secrets = secrets;
    }

    pub fn crypt_secrets(&self) -> &Secrets {
        &self.crypt_secrets
    }

    pub fn crypt_lock(&self) -> Lock {
        self.crypt_secrets.lock(self.passphrase_warn)
    }

    pub fn crypt_identity(&self, home: &Path) -> Identity {
        self.crypt_secrets.identity(home)
    }

    pub fn set_passphrase_warn(&mut self, warn: bool) {
        self.passphrase_warn = warn;
    }

    pub fn passphrase_warn(&self) -> bool {
        self.passphrase_warn
    }

    pub fn link(&self) -> LinkMode {
        self.link
    }

    pub fn conflict(&self) -> ConflictPolicy {
        self.conflict
    }

    pub fn rules(&self) -> &[Rule] {
        &self.rules
    }

    pub fn classes(&self) -> &[Class] {
        &self.classes
    }

    pub fn class(&self, name: &str) -> Option<&Class> {
        self.classes.iter().find(|class| class.name == name)
    }

    pub fn is_ignored(&self, relative: &Path) -> bool {
        if inside_git_dir(relative) {
            return true;
        }
        self.matching(relative)
            .filter_map(Rule::ignore)
            .next_back()
            .unwrap_or(false)
    }

    pub fn placement<'a>(&'a self, relative: &'a Path) -> Placement<'a> {
        Placement::new(self.link_mode(relative))
            .with_mode(self.mode(relative))
            .with_owner(self.owner(relative))
    }

    pub fn link_mode(&self, relative: &Path) -> LinkMode {
        self.matching(relative)
            .filter_map(|rule| rule.link)
            .next_back()
            .unwrap_or(self.link)
    }

    pub fn conflict_policy(&self, relative: &Path) -> ConflictPolicy {
        self.matching(relative)
            .filter_map(|rule| rule.conflict)
            .next_back()
            .unwrap_or(self.conflict)
    }

    pub fn on_change<'a>(&'a self, relative: &'a Path) -> Option<&'a str> {
        self.matching(relative)
            .filter_map(Rule::on_change)
            .next_back()
    }

    pub fn mode(&self, relative: &Path) -> Option<u32> {
        self.matching(relative).filter_map(Rule::mode).next_back()
    }

    pub fn owner<'a>(&'a self, relative: &'a Path) -> Option<&'a str> {
        self.matching(relative).filter_map(Rule::owner).next_back()
    }

    pub fn autocommit(&self, relative: &Path) -> bool {
        match self
            .matching(relative)
            .filter_map(Rule::autocommit)
            .next_back()
        {
            Some(autocommit) => autocommit,
            None => self.autocommit || self.autopush || self.pushed(relative),
        }
    }

    pub fn autopush(&self, relative: &Path) -> bool {
        self.autocommit(relative) && self.pushed(relative)
    }

    fn pushed(&self, relative: &Path) -> bool {
        self.matching(relative)
            .filter_map(Rule::autopush)
            .next_back()
            .unwrap_or(self.autopush)
    }

    pub fn encrypt(&self, relative: &Path) -> bool {
        self.matching(relative)
            .filter_map(Rule::encrypt)
            .next_back()
            .unwrap_or(false)
    }

    pub fn lfs_patterns(&self) -> Vec<(String, bool)> {
        if !self.lfs {
            return Vec::new();
        }

        self.rules
            .iter()
            .filter_map(|rule| rule.lfs().map(|tracked| (rule.pattern(), tracked)))
            .flat_map(|(pattern, tracked)| {
                globs(pattern).into_iter().map(move |glob| (glob, tracked))
            })
            .collect()
    }

    fn matching<'a>(&'a self, relative: &'a Path) -> impl DoubleEndedIterator<Item = &'a Rule> {
        self.rules
            .iter()
            .filter(move |rule| matches_path_or_ancestor(&rule.pattern, relative))
    }
}

impl Matcher {
    fn matches(&self, relative: &Path) -> bool {
        match self {
            Self::Glob(pattern) => pattern.matches_path_with(relative, MATCH),
            Self::Regex(regex) => regex.is_match(&relative.to_string_lossy()),
            Self::Any(matchers) => matchers.iter().any(|matcher| matcher.matches(relative)),
        }
    }
}

impl fmt::Display for Matcher {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Glob(pattern) => write!(formatter, "{pattern}"),
            Self::Regex(regex) => write!(formatter, "/{regex}/"),
            Self::Any(matchers) => {
                let joined: Vec<String> = matchers.iter().map(Self::to_string).collect();
                write!(formatter, "{{{}}}", joined.join(", "))
            }
        }
    }
}

impl Rule {
    pub fn new(pattern: Matcher, link: Option<LinkMode>, conflict: Option<ConflictPolicy>) -> Self {
        Self {
            pattern,
            link,
            conflict,
            on_change: None,
            ignore: None,
            mode: None,
            owner: None,
            encrypt: None,
            lfs: None,
            autocommit: None,
            autopush: None,
        }
    }

    pub fn with_on_change(mut self, on_change: Option<String>) -> Self {
        self.on_change = on_change;
        self
    }

    pub fn with_ignore(mut self, ignore: Option<bool>) -> Self {
        self.ignore = ignore;
        self
    }

    pub fn with_mode(mut self, mode: Option<u32>) -> Self {
        self.mode = mode;
        self
    }

    pub fn with_owner(mut self, owner: Option<String>) -> Self {
        self.owner = owner;
        self
    }

    pub fn with_encrypt(mut self, encrypt: Option<bool>) -> Self {
        self.encrypt = encrypt;
        self
    }

    pub fn with_lfs(mut self, lfs: Option<bool>) -> Self {
        self.lfs = lfs;
        self
    }

    pub fn with_autocommit(mut self, autocommit: Option<bool>) -> Self {
        self.autocommit = autocommit;
        self
    }

    pub fn with_autopush(mut self, autopush: Option<bool>) -> Self {
        self.autopush = autopush;
        self
    }

    pub fn pattern(&self) -> &Matcher {
        &self.pattern
    }

    pub fn link(&self) -> Option<LinkMode> {
        self.link
    }

    pub fn conflict(&self) -> Option<ConflictPolicy> {
        self.conflict
    }

    pub fn on_change(&self) -> Option<&str> {
        self.on_change.as_deref()
    }

    pub fn ignore(&self) -> Option<bool> {
        self.ignore
    }

    pub fn mode(&self) -> Option<u32> {
        self.mode
    }

    pub fn owner(&self) -> Option<&str> {
        self.owner.as_deref()
    }

    pub fn encrypt(&self) -> Option<bool> {
        self.encrypt
    }

    pub fn lfs(&self) -> Option<bool> {
        self.lfs
    }

    pub fn autocommit(&self) -> Option<bool> {
        self.autocommit
    }

    pub fn autopush(&self) -> Option<bool> {
        self.autopush
    }
}

impl Class {
    pub fn new(
        name: String,
        prompt: Option<String>,
        choices: Vec<String>,
        default: Option<String>,
    ) -> Self {
        Self {
            name,
            prompt,
            choices,
            default,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn question(&self) -> String {
        self.prompt
            .clone()
            .unwrap_or_else(|| format!("{CLASS_QUESTION} `{}`", self.name))
    }

    pub fn choices(&self) -> &[String] {
        &self.choices
    }

    pub fn default(&self) -> Option<&str> {
        self.default.as_deref()
    }
}

fn globs(matcher: &Matcher) -> Vec<String> {
    match matcher {
        Matcher::Glob(pattern) => vec![pattern.as_str().to_string()],
        Matcher::Any(matchers) => matchers.iter().flat_map(globs).collect(),
        Matcher::Regex(_) => Vec::new(),
    }
}

fn inside_git_dir(relative: &Path) -> bool {
    relative
        .components()
        .any(|component| component.as_os_str() == GIT_DIR)
}

fn matches_path_or_ancestor(pattern: &Matcher, relative: &Path) -> bool {
    std::iter::successors(Some(relative), |path| path.parent())
        .take_while(|path| !path.as_os_str().is_empty())
        .any(|path| pattern.matches(path))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn git_metadata_is_always_ignored() {
        let config = Config::default();

        assert!(config.is_ignored(Path::new(".git/config")));
        assert!(config.is_ignored(Path::new(".config/repo/.git/HEAD")));
        assert!(!config.is_ignored(Path::new(".gitconfig")));
    }

    #[test]
    fn a_placement_gathers_what_the_rules_say_about_a_path() {
        let config = crate::lua::from_source(
            r#"ld.rules({ match = ".ssh/**", link = "copy", mode = "0600", owner = "me:wheel" })"#,
        )
        .unwrap();

        let key = config.placement(Path::new(".ssh/id_ed25519"));
        assert_eq!(key.link(), LinkMode::Copy);
        assert_eq!(key.mode(), Some(0o600));
        assert_eq!(key.owner(), Some("me:wheel"));

        let other = config.placement(Path::new(".bashrc"));
        assert_eq!(other.link(), LinkMode::Hard);
        assert_eq!(other.mode(), None);
        assert_eq!(other.owner(), None);
    }

    #[test]
    fn a_regex_covers_a_subtree_through_its_ancestors() {
        let matcher = Matcher::Regex(Regex::new(r"^\.ssh$").unwrap());

        assert!(matches_path_or_ancestor(
            &matcher,
            Path::new(".ssh/keys/id_ed25519")
        ));
        assert!(!matches_path_or_ancestor(&matcher, Path::new(".sshrc")));
    }

    #[test]
    fn a_matcher_holding_alternatives_matches_any_of_them() {
        let matcher = Matcher::Any(vec![
            Matcher::Glob(Pattern::new("**/*.tmp").unwrap()),
            Matcher::Regex(Regex::new(r"\.sw[po]$").unwrap()),
        ]);

        assert!(matcher.matches(Path::new(".cache/build.tmp")));
        assert!(matcher.matches(Path::new(".vimrc.swp")));
        assert!(!matcher.matches(Path::new(".vimrc")));
    }
}
