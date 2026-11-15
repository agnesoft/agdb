use crate::CIError;
use crate::check::Check;
use crate::check::Step;
use crate::check::step;
use crate::config::NoFields;
use crate::config::Toolchain;
use crate::config::TypeConfig;
use crate::manifest::Manifest;
use crate::project::Project;
use crate::project_type::Details;
use crate::project_type::Hook;
use crate::project_type::ProjectType;
use crate::project_type::no_checks;
use crate::utilities;
use std::collections::BTreeMap;

const DEFAULT_CHECKS: [&str; 4] = ["format", "lint", "build", "test"];

/// Of Cargo and Shell projects, runs as their last check unless the checks place it.
const RUN: &str = "run";

/// A script file in the project directory, `<name>.sh` (bash) or `<name>.ps1` (PowerShell,
/// preferred on Windows), `:` is `_` in the file name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Script {
    pub(crate) name: String,
    pub(crate) file: String,
}

impl Script {
    /// `path_exists` takes repository paths.
    pub(crate) fn find(dir: &str, name: &str, path_exists: &dyn Fn(&str) -> bool) -> Option<Self> {
        let [sh, ps1] = files(name);
        let order = if cfg!(windows) { [ps1, sh] } else { [sh, ps1] };
        order
            .into_iter()
            .find(|file| path_exists(&format!("{dir}/{file}")))
            .map(|file| Script {
                name: name.to_string(),
                file,
            })
    }

    /// Runs in the project directory.
    pub(crate) fn step(&self, dir: &str) -> Step {
        if self.file.ends_with(".ps1") {
            step("pwsh", &["-NoProfile", "-File", &self.file], dir)
        } else {
            step(utilities::BASH, &[&self.file], dir)
        }
    }

    pub(crate) fn check(&self, dir: &str) -> Check {
        Check {
            name: self.name.clone(),
            steps: vec![self.step(dir)],
        }
    }
}

pub(crate) fn files(name: &str) -> [String; 2] {
    let base = name.replace(':', "_");
    [format!("{base}.sh"), format!("{base}.ps1")]
}

/// Appends the `run` script unless the checks already have it.
pub(crate) fn push_run(dir: &str, checks: &mut Vec<Script>, path_exists: &dyn Fn(&str) -> bool) {
    if !checks.iter().any(|c| c.name == RUN)
        && let Some(run) = Script::find(dir, RUN, path_exists)
    {
        checks.push(run);
    }
}

/// The hook scripts of a Cargo or Shell project.
#[derive(Debug, Default)]
pub(crate) struct Hooks(BTreeMap<Hook, Script>);

impl Hooks {
    pub(crate) fn find(dir: &str, path_exists: &dyn Fn(&str) -> bool) -> Self {
        Self(
            Hook::ALL
                .into_iter()
                .filter_map(|hook| Script::find(dir, hook.name(), path_exists).map(|s| (hook, s)))
                .collect(),
        )
    }

    pub(crate) fn steps(&self, dir: &str, hook: Hook) -> Vec<Step> {
        self.0.get(&hook).map(|s| s.step(dir)).into_iter().collect()
    }
}

/// A directory without a manifest, its checks and hooks are script files.
#[derive(Debug)]
pub(crate) struct Shell {
    checks: Vec<Script>,
    hooks: Hooks,
}

impl ProjectType for Shell {
    /// Shell projects are `agci.toml` entries without a manifest.
    fn manifests(_tracked: &[&str]) -> Result<Vec<Manifest>, CIError> {
        Ok(Vec::new())
    }

    fn default_checks() -> Vec<&'static str> {
        DEFAULT_CHECKS.to_vec()
    }

    fn new(
        dir: &str,
        config: TypeConfig,
        _details: Vec<Details>,
        path_exists: &dyn Fn(&str) -> bool,
    ) -> Result<Self, CIError> {
        let NoFields {} = config.fields()?;
        let mut checks = Vec::new();

        for name in &config.checks {
            match Script::find(dir, name, path_exists) {
                Some(script) => checks.push(script),
                None if config.inherited => {}
                None => {
                    let [sh, ps1] = files(name);
                    return Err(
                        format!("'{dir}' check '{name}' has neither '{sh}' nor '{ps1}'").into(),
                    );
                }
            }
        }

        push_run(dir, &mut checks, path_exists);

        if checks.is_empty() {
            return Err(no_checks(dir));
        }

        Ok(Self {
            checks,
            hooks: Hooks::find(dir, path_exists),
        })
    }

    fn toolchains(&self) -> Vec<Toolchain> {
        Vec::new()
    }

    fn inputs(&self) -> &'static [&'static str] {
        &[]
    }

    fn check_names(&self) -> Vec<String> {
        self.checks.iter().map(|c| c.name.clone()).collect()
    }

    fn checks(&self, project: &Project, _projects: &[Project], _root: &str) -> Vec<Check> {
        self.checks.iter().map(|c| c.check(&project.dir)).collect()
    }

    fn hook(&self, project: &Project, hook: Hook) -> Vec<Step> {
        self.hooks.steps(&project.dir, hook)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::tests::build;
    use crate::project::tests::command_lines;
    use crate::project::tests::config;
    use crate::project::tests::manifest;
    use crate::project_type::Kind;

    #[test]
    fn runs_script_files_in_the_project_directory() {
        let projects = build(
            vec![manifest(
                "server/image",
                Kind::Shell,
                vec![],
                Some(config(&["test", "test:functional", "smoke"], &[])),
            )],
            |p| {
                [
                    "server/image",
                    "server/image/test.sh",
                    "server/image/test_functional.ps1",
                    "server/image/smoke.sh",
                    "server/image/smoke.ps1",
                ]
                .contains(&p)
            },
        )
        .unwrap();
        let smoke = if cfg!(windows) {
            "server/image> pwsh -NoProfile -File smoke.ps1".to_string()
        } else {
            format!("server/image> {} smoke.sh", utilities::BASH)
        };

        assert_eq!(
            command_lines(&projects, "server/image"),
            vec![
                format!("server/image> {} test.sh", utilities::BASH),
                "server/image> pwsh -NoProfile -File test_functional.ps1".to_string(),
                smoke,
            ]
        );
    }
}
