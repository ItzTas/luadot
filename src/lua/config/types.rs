use std::fmt;
use std::path::{Path, PathBuf};

use glob::Pattern;
use mlua::{AppDataRefMut, Lua};
use regex::Regex;

use super::constants::{CLASS_QUESTION, GIT_DIR, MATCH};
use crate::files::{ConflictPolicy, LinkMode};

#[derive(Debug, Clone)]
pub struct Config {
    link: LinkMode,
    conflict: ConflictPolicy,
    rules: Vec<Rule>,
    classes: Vec<Class>,
    pkg_warn: bool,
    backup: bool,
    backup_dir: Option<PathBuf>,
    backup_keep: Option<u32>,
    repo_dir: Option<PathBuf>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            link: LinkMode::default(),
            conflict: ConflictPolicy::default(),
            rules: Vec::new(),
            classes: Vec::new(),
            pkg_warn: true,
            backup: true,
            backup_dir: None,
            backup_keep: None,
            repo_dir: None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Matcher {
    Glob(Pattern),
    Regex(Regex),
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Class {
    name: String,
    prompt: Option<String>,
    choices: Vec<String>,
    default: Option<String>,
}

impl Config {
    pub fn building(lua: &Lua) -> mlua::Result<AppDataRefMut<'_, Config>> {
        lua.app_data_mut::<Config>()
            .ok_or_else(|| mlua::Error::external("the configuration is not available"))
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

    pub fn set_repo_dir(&mut self, dir: PathBuf) {
        self.repo_dir = Some(dir);
    }

    pub fn repo_dir(&self) -> Option<&Path> {
        self.repo_dir.as_deref()
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

    pub fn encrypt(&self, relative: &Path) -> bool {
        self.matching(relative)
            .filter_map(Rule::encrypt)
            .next_back()
            .unwrap_or(false)
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
        }
    }
}

impl fmt::Display for Matcher {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Glob(pattern) => write!(formatter, "{pattern}"),
            Self::Regex(regex) => write!(formatter, "/{regex}/"),
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
    fn default_config_uses_the_default_mode_and_policy() {
        let config = Config::default();
        let path = Path::new(".bashrc");

        assert_eq!(config.link_mode(path), LinkMode::default());
        assert_eq!(config.conflict_policy(path), ConflictPolicy::default());
        assert!(!config.is_ignored(path));
        assert_eq!(config.mode(path), None);
        assert_eq!(config.owner(path), None);
        assert!(!config.encrypt(path));
    }

    #[test]
    fn git_metadata_is_always_ignored() {
        let config = Config::default();

        assert!(config.is_ignored(Path::new(".git/config")));
        assert!(config.is_ignored(Path::new(".config/repo/.git/HEAD")));
        assert!(!config.is_ignored(Path::new(".gitconfig")));
    }

    #[test]
    fn the_config_file_itself_can_be_managed() {
        let config = Config::default();

        assert!(!config.is_ignored(Path::new(".config/luadot/config.lua")));
    }

    #[test]
    fn a_regex_reads_the_path_as_a_string() {
        let matcher = Matcher::Regex(Regex::new(r"\.config/[^/]+/init\.lua$").unwrap());

        assert!(matcher.matches(Path::new(".config/nvim/init.lua")));
        assert!(!matcher.matches(Path::new(".config/nvim/lua/init.lua")));
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
    fn a_matcher_prints_the_syntax_it_was_written_in() {
        assert_eq!(
            Matcher::Glob(Pattern::new(".ssh/**").unwrap()).to_string(),
            ".ssh/**"
        );
        assert_eq!(
            Matcher::Regex(Regex::new(r"^\.ssh").unwrap()).to_string(),
            r"/^\.ssh/"
        );
    }

    #[test]
    fn default_config_declares_no_class() {
        let config = Config::default();

        assert!(config.classes().is_empty());
        assert!(config.class("form-factor").is_none());
    }

    #[test]
    fn classes_are_kept_in_the_order_they_are_declared() {
        let mut config = Config::default();
        config.add_class(Class::new(
            "form-factor".to_string(),
            None,
            Vec::new(),
            None,
        ));
        config.add_class(Class::new("email".to_string(), None, Vec::new(), None));

        let names: Vec<&str> = config.classes().iter().map(Class::name).collect();
        assert_eq!(names, ["form-factor", "email"]);
    }

    #[test]
    fn declaring_a_class_again_replaces_the_first_one() {
        let mut config = Config::default();
        config.add_class(Class::new(
            "form-factor".to_string(),
            Some("first".to_string()),
            Vec::new(),
            None,
        ));
        config.add_class(Class::new(
            "form-factor".to_string(),
            Some("second".to_string()),
            Vec::new(),
            None,
        ));

        assert_eq!(config.classes().len(), 1);
        assert_eq!(config.class("form-factor").unwrap().question(), "second");
    }

    #[test]
    fn a_class_without_a_prompt_asks_for_itself() {
        let class = Class::new("form-factor".to_string(), None, Vec::new(), None);

        assert_eq!(class.question(), "define the class `form-factor`");
    }
}
