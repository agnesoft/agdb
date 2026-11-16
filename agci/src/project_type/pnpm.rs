use crate::CIError;
use crate::check::Check;
use crate::check::Step;
use crate::check::step;
use crate::config::NoFields;
use crate::config::Toolchain;
use crate::config::TypeConfig;
use crate::manifest::Manifest;
use crate::manifest::nested;
use crate::project::Package;
use crate::project::Project;
use crate::project_type::Details;
use crate::project_type::Hook;
use crate::project_type::ProjectType;
use crate::project_type::Publish;
use crate::project_type::Registry;
use crate::project_type::RootChecks;
use crate::project_type::no_checks;
use crate::release::with_version;
use agplatform::Fs;
use agplatform::Platform;
use std::collections::BTreeMap;
use std::collections::BTreeSet;

mod package_json;

#[cfg(target_os = "windows")]
pub(crate) const PNPM: &str = "pnpm.cmd";
#[cfg(not(target_os = "windows"))]
pub(crate) const PNPM: &str = "pnpm";

/// Without the root package.json `check` script.
const DEFAULT_CHECKS: [&str; 6] = [
    "format:check",
    "build",
    "lint:check",
    "type-check",
    "test",
    "test:functional",
];

/// Changes to these affect every pnpm project.
pub(crate) const INPUTS: [&str; 6] = [
    "package.json",
    "pnpm-lock.yaml",
    "pnpm-workspace.yaml",
    "turbo.json",
    "vitest.workspace.ts",
    ".npmrc",
];

/// A package.json script of any package of the project.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Script(String);

/// Of a package.json.
#[derive(Debug)]
pub(crate) struct PackageDetails {
    name: String,
    /// The package.json.
    file: String,
    scripts: BTreeSet<String>,
    /// Required unless private.
    version: Option<String>,
}

/// package.json packages (pnpm workspace) run via turbo.
#[derive(Debug)]
pub(crate) struct Pnpm {
    checks: Vec<Script>,
    hooks: BTreeSet<Hook>,
    /// Of the project's packages by name.
    packages: BTreeMap<String, PackageDetails>,
}

impl ProjectType for Pnpm {
    fn manifests(platform: &impl Platform, tracked: &[&str]) -> Result<Vec<Manifest>, CIError> {
        nested(tracked, "package.json")
            .into_iter()
            .map(|(dir, file)| {
                package_json::from_package_json(dir, &platform.fs().read_to_string(file)?)
            })
            .collect()
    }

    fn default_checks() -> Vec<&'static str> {
        DEFAULT_CHECKS.to_vec()
    }

    /// The tasks of the root package.json `check` script.
    fn root_checks(read: &dyn Fn(&str) -> Result<Option<String>, CIError>) -> RootChecks {
        match read("package.json")? {
            Some(content) => package_json::root_checks(&content),
            None => Ok(None),
        }
    }

    fn setup() -> Vec<Step> {
        vec![install()]
    }

    fn new(
        dir: &str,
        config: TypeConfig,
        details: Vec<Details>,
        _path_exists: &dyn Fn(&str) -> bool,
    ) -> Result<Self, CIError> {
        let NoFields {} = config.fields()?;
        let packages: BTreeMap<String, PackageDetails> = details
            .into_iter()
            .filter_map(|d| match d {
                Details::Pnpm(d) => Some((d.name.clone(), d)),
                _ => None,
            })
            .collect();
        let scripts: BTreeSet<&str> = packages
            .values()
            .flat_map(|p| p.scripts.iter().map(String::as_str))
            .collect();
        let mut checks = Vec::new();

        for name in config.checks {
            if scripts.contains(name.as_str()) {
                checks.push(Script(name));
            } else if !config.inherited {
                return Err(format!(
                    "'{dir}' check '{name}' is not a script of any of its packages"
                )
                .into());
            }
        }

        if checks.is_empty() {
            return Err(no_checks(dir));
        }

        let hooks = Hook::ALL
            .into_iter()
            .filter(|h| scripts.contains(h.name()))
            .collect();
        Ok(Self {
            checks,
            hooks,
            packages,
        })
    }

    fn artifacts(&self) -> &'static [&'static str] {
        &["coverage/", "**/playwright-report/"]
    }

    fn toolchains(&self) -> Vec<Toolchain> {
        vec![Toolchain::Node]
    }

    fn inputs(&self) -> &'static [&'static str] {
        &INPUTS
    }

    fn check_names(&self) -> Vec<String> {
        self.checks.iter().map(|c| c.0.clone()).collect()
    }

    fn checks(&self, project: &Project, _projects: &[Project], _root: &str) -> Vec<Check> {
        let mut checks = vec![Check {
            name: "install".to_string(),
            steps: vec![install()],
        }];
        checks.extend(self.checks.iter().map(|c| Check {
            name: c.0.clone(),
            steps: vec![turbo(project, &c.0)],
        }));
        checks
    }

    fn hook(&self, project: &Project, hook: Hook) -> Vec<Step> {
        if self.hooks.contains(&hook) {
            vec![turbo(project, hook.name())]
        } else {
            Vec::new()
        }
    }

    fn publish(&self, package: &Package, version: &str) -> Option<Publish> {
        let details = self.packages.get(&package.name)?;
        let mut steps = Vec::new();

        if details.scripts.contains("build") {
            steps.push(build(&package.name));
        }

        steps.push(step(
            PNPM,
            &[
                "publish",
                "--provenance",
                "--access",
                "public",
                "--no-git-checks",
            ],
            &package.dir,
        ));
        Some(Publish {
            registry: Registry::Npm,
            version: details.version.clone().unwrap_or(version.to_string()),
            steps: with_version(steps, version),
        })
    }

    fn version_updates(
        &self,
        from: &str,
        to: &str,
        read: &dyn Fn(&str) -> Result<String, CIError>,
    ) -> Result<Vec<(String, String)>, CIError> {
        self.packages
            .values()
            .filter(|package| package.version.as_deref() == Some(from))
            .map(|package| {
                let content = read(&package.file)?.replace(
                    &format!("\"version\": \"{from}\""),
                    &format!("\"version\": \"{to}\""),
                );
                Ok((package.file.clone(), content))
            })
            .collect()
    }
}

fn install() -> Step {
    step(PNPM, &["install", "--frozen-lockfile"], "")
}

/// The script via turbo over all packages of the project (in the packages that define it).
fn turbo(project: &Project, script: &str) -> Step {
    let filters: Vec<String> = project
        .packages
        .iter()
        .map(|p| format!("--filter={p}"))
        .collect();
    let mut args = vec![
        "exec",
        "turbo",
        "run",
        script,
        "--force",
        "--env-mode=loose",
        // the default (`grouped` on CI) opens its own GitHub log groups, which do not nest in agci's
        "--log-order=stream",
    ];
    args.extend(filters.iter().map(String::as_str));
    step(PNPM, &args, "")
}

fn build(package: &str) -> Step {
    step(
        PNPM,
        &[
            "exec",
            "turbo",
            "run",
            "build",
            "--env-mode=loose",
            "--log-order=stream",
            &format!("--filter={package}"),
        ],
        "",
    )
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::project::tests::build;
    use crate::project::tests::command_lines;
    use crate::project::tests::config;
    use crate::project::tests::manifest as project_manifest;
    use crate::project_type::Kind;

    /// Of a package with the scripts, published with a version.
    pub(crate) fn package_details(name: &str, scripts: &[&str], version: Option<&str>) -> Details {
        Details::Pnpm(PackageDetails {
            name: name.to_string(),
            file: format!("{name}/package.json"),
            scripts: scripts.iter().map(|s| s.to_string()).collect(),
            version: version.map(str::to_string),
        })
    }

    #[test]
    fn runs_scripts_in_declared_order_over_all_packages() {
        let projects = build(
            vec![
                project_manifest(
                    "ui",
                    Kind::Pnpm,
                    vec![],
                    Some(config(&["test", "build"], &[])),
                ),
                project_manifest("ui/lib", Kind::Pnpm, vec![], None),
            ],
            |_| false,
        )
        .unwrap();

        assert_eq!(
            projects[0].project_type.check_names(),
            vec!["test", "build"]
        );
        assert_eq!(
            command_lines(&projects, "ui"),
            vec![
                format!("> {PNPM} install --frozen-lockfile"),
                format!(
                    "> {PNPM} exec turbo run test --force --env-mode=loose --log-order=stream --filter=ui --filter=ui_lib"
                ),
                format!(
                    "> {PNPM} exec turbo run build --force --env-mode=loose --log-order=stream --filter=ui --filter=ui_lib"
                ),
            ]
        );
    }
}
