use std::path::Path;

use glob::Pattern;
use mlua::{AppDataRefMut, Lua};

use super::constants::{CLASS_QUESTION, GIT_DIR, MATCH};
use crate::files::{ConflictPolicy, LinkMode};

#[derive(Debug, Clone)]
pub struct Config {
    link: LinkMode,
    conflict: ConflictPolicy,
    ignore: Vec<Pattern>,
    rules: Vec<Rule>,
    classes: Vec<Class>,
    pkg_warn: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            link: LinkMode::default(),
            conflict: ConflictPolicy::default(),
            ignore: Vec::new(),
            rules: Vec::new(),
            classes: Vec::new(),
            pkg_warn: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Rule {
    pattern: Pattern,
    link: Option<LinkMode>,
    conflict: Option<ConflictPolicy>,
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

    pub fn add_ignore(&mut self, patterns: Vec<Pattern>) {
        self.ignore.extend(patterns);
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

    pub fn link(&self) -> LinkMode {
        self.link
    }

    pub fn conflict(&self) -> ConflictPolicy {
        self.conflict
    }

    pub fn ignore(&self) -> &[Pattern] {
        &self.ignore
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
        self.ignore
            .iter()
            .any(|pattern| matches_path_or_ancestor(pattern, relative))
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

    fn matching<'a>(&'a self, relative: &'a Path) -> impl DoubleEndedIterator<Item = &'a Rule> {
        self.rules
            .iter()
            .filter(move |rule| matches_path_or_ancestor(&rule.pattern, relative))
    }
}

impl Rule {
    pub fn new(pattern: Pattern, link: Option<LinkMode>, conflict: Option<ConflictPolicy>) -> Self {
        Self {
            pattern,
            link,
            conflict,
        }
    }

    pub fn pattern(&self) -> &Pattern {
        &self.pattern
    }

    pub fn link(&self) -> Option<LinkMode> {
        self.link
    }

    pub fn conflict(&self) -> Option<ConflictPolicy> {
        self.conflict
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

fn matches_path_or_ancestor(pattern: &Pattern, relative: &Path) -> bool {
    std::iter::successors(Some(relative), |path| path.parent())
        .take_while(|path| !path.as_os_str().is_empty())
        .any(|path| pattern.matches_path_with(path, MATCH))
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

        assert!(!config.is_ignored(Path::new(".config/luadot/ld.lua")));
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
