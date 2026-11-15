use crate::CIError;
use crate::check::Check;
use crate::check::Step;
use crate::check::step;
use crate::config::Toolchain;
use crate::config::TypeConfig;
use crate::manifest::Manifest;
use crate::project::Package;
use crate::project::Project;
use crate::project_type::Details;
use crate::project_type::Hook;
use crate::project_type::Kind;
use crate::project_type::ProjectType;
use crate::project_type::Publish;
use crate::project_type::no_checks;
use crate::project_type::shell::Hooks;
use crate::project_type::shell::Script;
use crate::project_type::shell::files;
use crate::project_type::shell::push_run;
use crate::utilities;
use registry::CratesIo;
use serde::Deserialize;
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::process::Command;

mod metadata;
mod registry;

/// Changes to these affect every Cargo project. Entries ending with `/` are directories.
pub(crate) const INPUTS: [&str; 5] = [
    "Cargo.toml",
    "Cargo.lock",
    "rust-toolchain",
    "rust-toolchain.toml",
    ".cargo/",
];

/// The checks agci runs itself, always in this order (before the scripts).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Builtin {
    Format,
    Lint,
    Testcheck,
    Doc,
    Examples,
    Bench,
    Coverage,
}

/// Without `agci.toml` checks, those the crate cannot run are skipped.
const DEFAULT_CHECKS: [Builtin; 6] = [
    Builtin::Format,
    Builtin::Lint,
    Builtin::Doc,
    Builtin::Examples,
    Builtin::Bench,
    Builtin::Coverage,
];

impl Builtin {
    const ALL: [Builtin; 7] = [
        Builtin::Format,
        Builtin::Lint,
        Builtin::Testcheck,
        Builtin::Doc,
        Builtin::Examples,
        Builtin::Bench,
        Builtin::Coverage,
    ];

    /// In `agci.toml`.
    pub(crate) fn name(self) -> &'static str {
        match self {
            Builtin::Format => "format",
            Builtin::Lint => "lint",
            Builtin::Testcheck => "testcheck",
            Builtin::Doc => "doc",
            Builtin::Examples => "examples",
            Builtin::Bench => "bench",
            Builtin::Coverage => "coverage",
        }
    }

    fn parse(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|b| b.name() == name)
    }
}

/// The crate's targets some built-in checks need.
#[derive(Clone, Copy, Debug, Default)]
struct Targets {
    library: bool,
    examples: bool,
    benches: bool,
}

/// Of the crate's manifest.
#[derive(Debug)]
pub(crate) struct CrateDetails {
    targets: Targets,
}

impl Targets {
    /// What the check needs that the crate does not have.
    fn missing(self, check: Builtin) -> Option<&'static str> {
        match check {
            Builtin::Doc if !self.library => Some("library"),
            Builtin::Examples if !self.examples => Some("example targets"),
            Builtin::Bench if !self.benches => Some("bench targets"),
            _ => None,
        }
    }
}

/// The fields of cargo projects in `agci.toml`.
#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct Fields {
    /// The `coverage` check requires the project's own tests to cover all of its lines, regions
    /// and functions (default), `false` only runs the tests (the check is named `test`).
    coverage: Option<bool>,
    /// Additionally lint each combination with `--no-default-features`.
    feature_combinations: Vec<String>,
}

/// A Cargo workspace member.
#[derive(Debug, Default)]
pub(crate) struct Cargo {
    builtins: BTreeSet<Builtin>,
    scripts: Vec<Script>,
    hooks: Hooks,
    /// The `coverage` check requires full coverage, otherwise it only runs the tests.
    coverage: bool,
    feature_combinations: Vec<String>,
}

impl Cargo {
    /// The name it runs as: `coverage` without coverage is `test`.
    fn name(&self, check: Builtin) -> &'static str {
        if check == Builtin::Coverage && !self.coverage {
            "test"
        } else {
            check.name()
        }
    }

    fn steps(
        &self,
        check: Builtin,
        project: &Project,
        projects: &[Project],
        root: &str,
    ) -> Vec<Step> {
        let name = project.packages[0].as_str();
        let features = ["--all-features"];
        let cargo = |args: &[&str], extra: &[&str]| {
            let mut all = args.to_vec();
            all.extend(extra);
            step("cargo", &all, "")
        };

        match check {
            Builtin::Format => vec![cargo(&["fmt", "-p", name, "--check"], &[])],
            Builtin::Lint => {
                let mut steps = vec![cargo(
                    &[&["clippy", "-p", name, "--all-targets"], &features[..]].concat(),
                    &["--", "-D", "warnings"],
                )];

                for combination in &self.feature_combinations {
                    for targets in ["--lib", "--all-targets"] {
                        let mut args = vec!["clippy", "-p", name, targets, "--no-default-features"];

                        if !combination.is_empty() {
                            args.extend(["--features", combination]);
                        }

                        steps.push(cargo(&args, &["--", "-D", "warnings"]));
                    }
                }

                steps
            }
            Builtin::Testcheck => vec![step(
                "cargo",
                &["run", "-r", "-p", "agtestcheck"],
                &project.dir,
            )],
            Builtin::Doc => vec![cargo(
                &[&["test", "-p", name], &features[..]].concat(),
                &["--doc"],
            )],
            Builtin::Examples => vec![cargo(
                &[&["test", "-p", name, "--examples"], &features[..]].concat(),
                &[],
            )],
            Builtin::Bench => vec![cargo(
                &[&["bench", "-p", name, "--bench", "*"], &features[..]].concat(),
                &["--", "--samples", "1", "--iters", "1"],
            )],
            Builtin::Coverage if !self.coverage => {
                vec![cargo(&[&["test", "-p", name], &features[..]].concat(), &[])]
            }
            Builtin::Coverage => {
                let ignore = coverage_ignore(project, projects, root);
                vec![
                    cargo(&["llvm-cov", "clean", "--workspace"], &[]),
                    cargo(
                        &[&["llvm-cov", "-p", name], &features[..]].concat(),
                        &[
                            "--show-missing-lines",
                            "--ignore-filename-regex",
                            &ignore,
                            "--fail-uncovered-lines",
                            "0",
                            "--fail-uncovered-regions",
                            "0",
                            "--fail-uncovered-functions",
                            "0",
                        ],
                    ),
                ]
            }
        }
    }
}

impl ProjectType for Cargo {
    fn manifests(tracked: &[&str]) -> Result<Vec<Manifest>, CIError> {
        metadata::manifests(tracked)
    }

    fn default_checks() -> Vec<&'static str> {
        DEFAULT_CHECKS.iter().map(|c| c.name()).collect()
    }

    /// `[workspace.package]` or `[package]` of the root `Cargo.toml`.
    fn version() -> Result<Option<String>, CIError> {
        let manifest: toml::Table = match std::fs::read_to_string("Cargo.toml") {
            Ok(content) => toml::from_str(&content).map_err(|e| format!("'Cargo.toml': {e}"))?,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(e.into()),
        };
        let package = manifest
            .get("workspace")
            .and_then(|w| w.get("package"))
            .or(manifest.get("package"));
        Ok(package
            .and_then(|p| p.get("version"))
            .and_then(|v| v.as_str())
            .map(str::to_string))
    }

    fn new(
        dir: &str,
        config: TypeConfig,
        details: Vec<Details>,
        path_exists: &dyn Fn(&str) -> bool,
    ) -> Result<Self, CIError> {
        let fields: Fields = config.fields()?;
        // the project's own crate comes first, nested ones fold into it
        let targets = details
            .into_iter()
            .find_map(|d| match d {
                Details::Cargo(d) => Some(d.targets),
                _ => None,
            })
            .unwrap_or_default();
        let mut builtins = BTreeSet::new();
        let mut scripts = Vec::new();

        for name in &config.checks {
            if let Some(builtin) = Builtin::parse(name) {
                match targets.missing(builtin) {
                    None => {
                        builtins.insert(builtin);
                    }
                    Some(_) if config.inherited => {}
                    Some(missing) => {
                        return Err(
                            format!("'{dir}' has the '{name}' check but no {missing}").into()
                        );
                    }
                }
            } else if let Some(script) = Script::find(dir, name, path_exists) {
                scripts.push(script);
            } else if !config.inherited {
                let [sh, ps1] = files(name);
                return Err(format!(
                    "'{dir}' check '{name}' is neither one of {:?} nor has '{sh}' or '{ps1}'",
                    Builtin::ALL.map(Builtin::name)
                )
                .into());
            }
        }

        push_run(dir, &mut scripts, path_exists);

        if builtins.is_empty() && scripts.is_empty() {
            return Err(no_checks(dir));
        }

        if !fields.feature_combinations.is_empty() && !builtins.contains(&Builtin::Lint) {
            return Err(
                format!("'{dir}' sets feature_combinations without the 'lint' check").into(),
            );
        }

        Ok(Self {
            builtins,
            scripts,
            hooks: Hooks::find(dir, path_exists),
            coverage: fields.coverage.unwrap_or(true),
            feature_combinations: fields.feature_combinations,
        })
    }

    fn toolchains(&self) -> Vec<Toolchain> {
        if self.coverage && self.builtins.contains(&Builtin::Coverage) {
            vec![Toolchain::Rust, Toolchain::Coverage]
        } else {
            vec![Toolchain::Rust]
        }
    }

    fn inputs(&self) -> &'static [&'static str] {
        &INPUTS
    }

    fn check_names(&self) -> Vec<String> {
        self.builtins
            .iter()
            .map(|b| self.name(*b).to_string())
            .chain(self.scripts.iter().map(|s| s.name.clone()))
            .collect()
    }

    fn checks(&self, project: &Project, projects: &[Project], root: &str) -> Vec<Check> {
        self.builtins
            .iter()
            .map(|b| Check {
                name: self.name(*b).to_string(),
                steps: self.steps(*b, project, projects, root),
            })
            .chain(self.scripts.iter().map(|s| s.check(&project.dir)))
            .collect()
    }

    fn hook(&self, project: &Project, hook: Hook) -> Vec<Step> {
        self.hooks.steps(&project.dir, hook)
    }

    fn publish(&self, package: &Package, version: &str) -> Option<Publish> {
        Some(Publish {
            registry: &CratesIo,
            version: version.to_string(),
            steps: vec![step("cargo", &["publish", "-p", &package.name], "")],
        })
    }
}

/// Builds the binary of the package in release mode, its path.
pub(crate) fn build_release(package: &str, exe: &str) -> Result<PathBuf, CIError> {
    utilities::stream_command(Command::new("cargo").args(["build", "-r", "-p", package]))?;
    Ok(metadata::target_dir()?.join("release").join(exe))
}

/// Coverage counts only the project's own sources: other Cargo projects and its examples are ignored.
fn coverage_ignore(project: &Project, projects: &[Project], root: &str) -> String {
    let sep = r"[/\\]";
    let path = |p: &str| p.split('/').map(regex_escape).collect::<Vec<_>>().join(sep);
    let mut ignored: Vec<String> = projects
        .iter()
        .filter(|p| {
            p.kind == Kind::Cargo
                && p.dir != project.dir
                && !project.owns(&p.dir)
                && !p.owns(&project.dir)
        })
        .map(|p| path(&p.dir))
        .collect();
    ignored.push(format!("{}{sep}examples", path(&project.dir)));

    format!(
        "^{}{sep}({}){sep}",
        path(&root.replace('\\', "/")),
        ignored.join("|")
    )
}

fn regex_escape(text: &str) -> String {
    text.chars()
        .flat_map(|c| {
            if r"\.+*?()|[]{}^$".contains(c) {
                vec!['\\', c]
            } else {
                vec![c]
            }
        })
        .collect()
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::config::Config;
    use crate::manifest::Dep;
    use crate::project::tests::Entry;
    use crate::project::tests::build;
    use crate::project::tests::command_lines;
    use crate::project::tests::config;
    use crate::project::tests::manifest;
    use crate::utilities;

    /// `agdb_server` depends on `agdb`, the server has a `smoke.sh` script.
    fn lines(dir: &str, edit: impl FnOnce(&mut Config)) -> Vec<String> {
        let mut entry = config(&[], &[]);
        edit(&mut entry);
        let (agdb, server) = if dir == "agdb" {
            (entry, config(&["coverage"], &[]))
        } else {
            (config(&["coverage"], &[]), entry)
        };
        let all_targets = |mut entry: Entry| {
            entry.0.as_mut().unwrap().details = crate_details(true, true, true);
            entry
        };
        let projects = build(
            vec![
                all_targets(manifest("agdb", Kind::Cargo, vec![], Some(agdb))),
                all_targets(manifest(
                    "agdb_server",
                    Kind::Cargo,
                    vec![Dep::Dir("agdb".to_string())],
                    Some(server),
                )),
            ],
            |p| p == "agdb_server/smoke.sh",
        )
        .unwrap();
        command_lines(&projects, dir)
    }

    fn names(edit: impl FnOnce(&mut Config)) -> Vec<String> {
        let mut entry = config(&[], &[]);
        edit(&mut entry);
        build(
            vec![manifest("agdb", Kind::Cargo, vec![], Some(entry))],
            |p| p == "agdb/regenerate.ps1",
        )
        .unwrap()[0]
            .project_type
            .check_names()
    }

    /// Of a crate with the targets.
    pub(crate) fn crate_details(library: bool, examples: bool, benches: bool) -> Details {
        Details::Cargo(CrateDetails {
            targets: Targets {
                library,
                examples,
                benches,
            },
        })
    }

    fn checks(names: &[&str]) -> Vec<String> {
        names.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn builtins_run_in_their_order_then_the_scripts() {
        assert_eq!(
            names(|c| c.checks = checks(&["regenerate", "coverage", "lint"])),
            vec!["lint", "coverage", "regenerate"]
        );
        assert_eq!(
            names(|c| {
                c.checks = checks(&["coverage"]);
                c.fields.insert("coverage".to_string(), false.into());
            }),
            vec!["test"]
        );
    }

    #[test]
    fn runs_checks_in_canonical_order() {
        assert_eq!(
            lines("agdb_server", |c| {
                c.checks = checks(&[
                    "smoke", "coverage", "bench", "examples", "doc", "lint", "format",
                ]);
                c.fields.insert("coverage".to_string(), false.into());
            }),
            vec![
                "> cargo fmt -p agdb_server --check".to_string(),
                "> cargo clippy -p agdb_server --all-targets --all-features -- -D warnings"
                    .to_string(),
                "> cargo test -p agdb_server --all-features --doc".to_string(),
                "> cargo test -p agdb_server --examples --all-features".to_string(),
                "> cargo bench -p agdb_server --bench * --all-features -- --samples 1 --iters 1"
                    .to_string(),
                "> cargo test -p agdb_server --all-features".to_string(),
                format!("agdb_server> {} smoke.sh", utilities::BASH),
            ]
        );
    }

    #[test]
    fn feature_combinations() {
        assert_eq!(
            lines("agdb", |c| {
                c.checks = checks(&["lint"]);
                c.fields.insert(
                    "feature_combinations".to_string(),
                    vec!["", "fs,env"].into(),
                );
            }),
            vec![
                "> cargo clippy -p agdb --all-targets --all-features -- -D warnings",
                "> cargo clippy -p agdb --lib --no-default-features -- -D warnings",
                "> cargo clippy -p agdb --all-targets --no-default-features -- -D warnings",
                "> cargo clippy -p agdb --lib --no-default-features --features fs,env -- -D warnings",
                "> cargo clippy -p agdb --all-targets --no-default-features --features fs,env -- -D warnings",
            ]
        );
    }

    #[test]
    fn full_coverage_ignores_other_projects() {
        let sep = r"[/\\]";

        assert_eq!(
            lines("agdb", |c| c.checks = checks(&["coverage"])),
            vec![
                "> cargo llvm-cov clean --workspace".to_string(),
                format!(
                    "> cargo llvm-cov -p agdb --all-features --show-missing-lines --ignore-filename-regex ^{sep}repo{sep}(agdb_server|agdb{sep}examples){sep} --fail-uncovered-lines 0 --fail-uncovered-regions 0 --fail-uncovered-functions 0"
                ),
            ]
        );
    }

    #[test]
    fn regex_escape_special_characters() {
        assert_eq!(regex_escape("a.b-c(d)"), r"a\.b-c\(d\)");
    }

    #[test]
    fn builtin_names_round_trip() {
        for builtin in Builtin::ALL {
            assert_eq!(Builtin::parse(builtin.name()), Some(builtin));
        }
        assert_eq!(Builtin::parse("fmt"), None);
    }

    #[test]
    fn coverage_is_measured_only_with_the_coverage_check() {
        let measures = |edit: fn(&mut Config)| {
            let mut entry = config(&[], &[]);
            edit(&mut entry);
            build(
                vec![manifest("a", Kind::Cargo, vec![], Some(entry))],
                |_| false,
            )
            .unwrap()
            .remove(0)
            .project_type
            .toolchains()
            .contains(&Toolchain::Coverage)
        };

        assert!(measures(|c| c.checks = checks(&["coverage"])));
        assert!(!measures(|c| {
            c.checks = checks(&["coverage"]);
            c.fields.insert("coverage".to_string(), false.into());
        }));
        assert!(!measures(|c| c.checks = checks(&["lint"])));
    }

    #[test]
    fn script_steps_run_in_the_project_directory() {
        assert_eq!(
            lines("agdb_server", |c| c.checks = checks(&["smoke"])),
            vec![format!("agdb_server> {} smoke.sh", utilities::BASH)]
        );
    }
}
