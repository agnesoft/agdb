use crate::CIError;
use crate::manifest::Kind;
use serde::Deserialize;
use serde::Serialize;
use std::collections::BTreeMap;

/// The configuration of the projects in the repository root, optional.
pub(crate) const FILE: &str = "agci.toml";

/// `agci.toml`: defaults per language and the entries of the projects by directory.
#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct File {
    #[serde(default)]
    pub(crate) defaults: Defaults,
    #[serde(default)]
    pub(crate) projects: BTreeMap<String, toml::Table>,
    /// The tasks of the root package.json `check` script, the built-in pnpm checks if set.
    #[serde(skip)]
    pub(crate) pnpm_checks: Option<Vec<String>>,
}

/// Fields of the projects of the language that they do not set themselves.
#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Defaults {
    cargo: Option<toml::Table>,
    pnpm: Option<toml::Table>,
    shell: Option<toml::Table>,
}

/// `[projects."<dir>"]` (or `[defaults.<language>]`) in `agci.toml`.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct Config {
    /// Cargo: built-in checks (see `CARGO_CHECKS`) in their order, then the others. pnpm: package.json
    /// scripts. Shell (and the other Cargo ones): `<check>.sh` or `<check>.ps1` in the project
    /// directory. Except the Cargo built-ins they run in the given order.
    pub(crate) checks: Vec<String>,
    /// Project directories or repository paths that affect this project in addition to the manifest dependencies.
    pub(crate) dependencies: Vec<String>,
    /// Defaults to `["linux"]`.
    pub(crate) os: Vec<Os>,
    /// Toolchains needed in addition to the project's own language.
    pub(crate) add_toolchains: Vec<Toolchain>,
    /// Projects of the same group are checked in one job (per OS), one after another.
    pub(crate) group: Option<String>,
    /// Cargo only: the `coverage` check requires the project's own tests to cover all of its lines,
    /// regions and functions (default), `false` only runs the tests (the check is named `test`).
    pub(crate) coverage: Option<bool>,
    /// Cargo only: additionally lint each combination with `--no-default-features`.
    pub(crate) feature_combinations: Vec<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Os {
    Linux,
    Windows,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Toolchain {
    Node,
    Php,
    Rust,
}

pub(crate) const CARGO_CHECKS: [&str; 7] = [
    "format",
    "lint",
    "testcheck",
    "doc",
    "examples",
    "bench",
    "coverage",
];

impl Config {
    pub(crate) fn os(&self) -> Vec<Os> {
        if self.os.is_empty() {
            vec![Os::Linux]
        } else {
            self.os.clone()
        }
    }

    pub(crate) fn coverage(&self) -> bool {
        self.coverage.unwrap_or(true)
    }

    pub(crate) fn has_check(&self, name: &str) -> bool {
        self.checks.iter().any(|c| c == name)
    }

    pub(crate) fn has_rust_only_fields(&self) -> bool {
        self.coverage.is_some() || !self.feature_combinations.is_empty()
    }
}

/// When neither `agci.toml` nor the project set them. Only those the project can run apply: Cargo
/// `doc` for crates with a library, `examples` and `bench` for crates with such targets, pnpm and
/// Shell ones if the project has the script.
fn builtin(kind: Kind) -> toml::Table {
    let checks: &[&str] = match kind {
        Kind::Cargo => &["format", "lint", "doc", "examples", "bench", "coverage"],
        Kind::Pnpm => &[
            "format:check",
            "build",
            "lint:check",
            "type-check",
            "test",
            "test:functional",
        ],
        Kind::Shell => &["format", "lint", "build", "test"],
    };
    toml::Table::from_iter([(
        "checks".to_string(),
        toml::Value::Array(checks.iter().map(|c| toml::Value::from(*c)).collect()),
    )])
}

impl File {
    pub(crate) fn parse(content: &str) -> Result<Self, CIError> {
        toml::from_str(content).map_err(|e| format!("'{FILE}': {e}").into())
    }

    /// The fields the project sets over the defaults of its language over the built-in ones. Only
    /// what is written counts, so a new field with a default does not change any project.
    pub(crate) fn effective(&self, dir: &str, kind: Kind) -> toml::Table {
        let defaults = match kind {
            Kind::Cargo => self.defaults.cargo.as_ref(),
            Kind::Pnpm => self.defaults.pnpm.as_ref(),
            Kind::Shell => self.defaults.shell.as_ref(),
        };
        let mut table = builtin(kind);

        if let (Kind::Pnpm, Some(checks)) = (kind, &self.pnpm_checks) {
            table.insert("checks".to_string(), toml::Value::from(checks.clone()));
        }

        for layer in [defaults, self.projects.get(dir)].into_iter().flatten() {
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
        assert_eq!(agdb.coverage, Some(true));
        assert_eq!(agdb.os, vec![Os::Linux, Os::Windows]);
        assert!(!inherited);

        let (detected, inherited) = config(content, "agdb_derive", Kind::Cargo).unwrap();
        assert_eq!(
            detected.checks,
            vec!["format", "lint", "doc", "examples", "bench", "coverage"]
        );
        assert_eq!(detected.coverage, Some(false));
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
    fn root_check_script_over_builtin_node_checks() {
        let mut file = File::parse("[projects.b]\nchecks = [\"test\"]").unwrap();
        file.pnpm_checks = Some(vec!["openapi".to_string(), "test".to_string()]);

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
        let error = config("[projects.a]\ncheck = []", "a", Kind::Cargo)
            .unwrap_err()
            .description;

        assert!(error.contains("project 'a'"), "{error}");
        assert!(error.contains("unknown field `check`"), "{error}");
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
                .contains("unknown field `php`")
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
