use crate::CIError;
use crate::project_type::Kind;
use serde::Deserialize;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::collections::BTreeMap;

/// The configuration of the projects in the repository root, optional.
pub(crate) const FILE: &str = "agci.toml";

/// `agci.toml`: defaults per project type and the entries of the projects by directory.
#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct File {
    /// Fields of the projects of the type that they do not set themselves.
    #[serde(default)]
    pub(crate) defaults: BTreeMap<Kind, toml::Table>,
    #[serde(default)]
    pub(crate) projects: BTreeMap<String, toml::Table>,
    /// The default checks the repository root sets per type, over the built-in ones.
    #[serde(skip)]
    pub(crate) root_checks: BTreeMap<Kind, Vec<String>>,
}

/// `[projects."<dir>"]` (or `[defaults.<type>]`) in `agci.toml`.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub(crate) struct Config {
    /// What they are is up to the project type, those of the defaults only if the project can run
    /// them.
    pub(crate) checks: Vec<String>,
    /// Project directories or repository paths that affect this project in addition to the manifest dependencies.
    pub(crate) dependencies: Vec<String>,
    /// Defaults to `["linux"]`.
    pub(crate) os: Vec<Os>,
    /// Toolchains needed in addition to the project's own type.
    pub(crate) add_toolchains: Vec<Toolchain>,
    /// Projects of the same group are checked in one job (per OS), one after another.
    pub(crate) group: Option<String>,
    /// Those of the project type.
    #[serde(flatten)]
    pub(crate) fields: toml::Table,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Os {
    Linux,
    Windows,
}

/// Installed in a job, in the order of the job matrix fields.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Toolchain {
    Rust,
    Node,
    Php,
    /// `cargo llvm-cov`.
    Coverage,
}

impl Toolchain {
    pub(crate) const ALL: [Toolchain; 4] = [
        Toolchain::Rust,
        Toolchain::Node,
        Toolchain::Php,
        Toolchain::Coverage,
    ];
}

impl Config {
    /// The settings of the project and what its type resolves: the checks (`inherited` if not
    /// its own) and its fields.
    pub(crate) fn split(self, dir: &str, inherited: bool) -> (Settings, TypeConfig) {
        (
            Settings {
                dependencies: self.dependencies,
                os: self.os,
                add_toolchains: self.add_toolchains,
                group: self.group,
            },
            TypeConfig {
                dir: dir.to_string(),
                checks: self.checks,
                inherited,
                fields: self.fields,
            },
        )
    }
}

/// The configuration of a project that does not depend on its type.
#[derive(Clone, Debug, Default)]
pub(crate) struct Settings {
    pub(crate) dependencies: Vec<String>,
    pub(crate) os: Vec<Os>,
    pub(crate) add_toolchains: Vec<Toolchain>,
    pub(crate) group: Option<String>,
}

impl Settings {
    pub(crate) fn os(&self) -> Vec<Os> {
        if self.os.is_empty() {
            vec![Os::Linux]
        } else {
            self.os.clone()
        }
    }
}

/// The part of a project's configuration its type resolves.
#[derive(Debug, Default)]
pub(crate) struct TypeConfig {
    dir: String,
    pub(crate) checks: Vec<String>,
    /// The checks are the defaults: those the project cannot run are skipped, not an error.
    pub(crate) inherited: bool,
    fields: toml::Table,
}

impl TypeConfig {
    /// The fields of the type, unknown ones are an error.
    pub(crate) fn fields<T: DeserializeOwned>(&self) -> Result<T, CIError> {
        toml::Value::Table(self.fields.clone())
            .try_into()
            .map_err(|e| format!("'{FILE}' project '{}': {e}", self.dir).into())
    }
}

/// The type has no fields besides the common ones.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct NoFields {}

impl File {
    pub(crate) fn parse(content: &str) -> Result<Self, CIError> {
        toml::from_str(content).map_err(|e| format!("'{FILE}': {e}").into())
    }

    /// The fields the project sets over the defaults of its type over the root and built-in
    /// checks. Only what is written counts, so a new field with a default does not change any
    /// project.
    pub(crate) fn effective(&self, dir: &str, kind: Kind) -> toml::Table {
        let checks = match self.root_checks.get(&kind) {
            Some(checks) => toml::Value::from(checks.clone()),
            None => toml::Value::from(kind.default_checks()),
        };
        let mut table = toml::Table::from_iter([("checks".to_string(), checks)]);

        for layer in [self.defaults.get(&kind), self.projects.get(dir)]
            .into_iter()
            .flatten()
        {
            table.extend(layer.clone());
        }

        table
    }

    /// The configuration of the project and whether its checks are not its own (defaults).
    pub(crate) fn config(&self, dir: &str, kind: Kind) -> Result<(Config, bool), CIError> {
        let config = toml::Value::Table(self.effective(dir, kind))
            .try_into()
            .map_err(|e| format!("'{FILE}' project '{dir}': {e}"))?;
        let inherited = !self
            .projects
            .get(dir)
            .is_some_and(|entry| entry.contains_key("checks"));
        Ok((config, inherited))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config(content: &str, dir: &str, kind: Kind) -> Result<(Config, bool), CIError> {
        File::parse(content)?.config(dir, kind)
    }

    #[test]
    fn project_fields_over_defaults_over_builtin() {
        let content = r#"
            [defaults.cargo]
            os = ["linux", "windows"]
            coverage = false

            [defaults.shell]
            checks = ["test"]

            [projects.agdb]
            checks = ["format", "coverage"]
            coverage = true

            [projects."agdb_server/image"]
            dependencies = ["agdb_server"]
            "#;

        let (agdb, inherited) = config(content, "agdb", Kind::Cargo).unwrap();
        assert_eq!(agdb.checks, vec!["format", "coverage"]);
        assert_eq!(agdb.fields["coverage"], toml::Value::from(true));
        assert_eq!(agdb.os, vec![Os::Linux, Os::Windows]);
        assert!(!inherited);

        let (detected, inherited) = config(content, "agdb_derive", Kind::Cargo).unwrap();
        assert_eq!(
            detected.checks,
            vec!["format", "lint", "doc", "examples", "bench", "coverage"]
        );
        assert_eq!(detected.fields["coverage"], toml::Value::from(false));
        assert!(inherited);

        let (node, _) = config(content, "web", Kind::Pnpm).unwrap();
        assert_eq!(
            node.checks,
            vec![
                "format:check",
                "build",
                "lint:check",
                "type-check",
                "test",
                "test:functional"
            ]
        );
        assert!(node.os.is_empty());

        let (image, inherited) = config(content, "agdb_server/image", Kind::Shell).unwrap();
        assert_eq!(image.checks, vec!["test"]);
        assert_eq!(image.dependencies, vec!["agdb_server"]);
        assert!(inherited);
    }

    #[test]
    fn root_checks_over_builtin_ones() {
        let mut file = File::parse("[projects.b]\nchecks = [\"test\"]").unwrap();
        file.root_checks
            .insert(Kind::Pnpm, vec!["openapi".to_string(), "test".to_string()]);

        assert_eq!(
            file.config("a", Kind::Pnpm).unwrap().0.checks,
            vec!["openapi", "test"]
        );
        assert_eq!(file.config("b", Kind::Pnpm).unwrap().0.checks, vec!["test"]);
        assert_eq!(
            file.config("c", Kind::Cargo).unwrap().0.checks,
            vec!["format", "lint", "doc", "examples", "bench", "coverage"]
        );
    }

    #[test]
    fn unknown_fields_fail() {
        assert!(
            File::parse("[project.a]\nchecks = []")
                .unwrap_err()
                .description
                .contains("unknown field `project`")
        );
        assert!(
            File::parse("[defaults.php]\nchecks = []")
                .unwrap_err()
                .description
                .contains("unknown variant `php`")
        );
    }

    #[test]
    fn effective_entries_ignore_order_and_formatting() {
        let effective = |content: &str| {
            serde_json::to_string(&File::parse(content).unwrap().effective("a", Kind::Cargo))
                .unwrap()
        };
        let a = effective("[projects.a]\nchecks = [\"test\"]\nos = [\"linux\"]");
        let b = effective("[projects.a]\nos = [ \"linux\" ] # comment\nchecks = [\"test\"]");
        let c = effective("[projects.a]\nchecks = [\"test\", \"lint\"]");

        assert_eq!(a, b);
        assert_ne!(a, c);
    }
}
