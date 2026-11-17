use crate::CIError;
use crate::affected;
use crate::affected::Snapshot;
use crate::git;
use crate::project::Project;
use crate::proof;
use crate::proof::Proofs;
use crate::report::Report;
use crate::report::Task;
use crate::utilities;
use agplatform::Platform;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::path::Path;
use std::process::Command;

#[derive(Debug, PartialEq)]
pub(crate) struct Step {
    pub(crate) program: String,
    pub(crate) args: Vec<String>,
    /// Relative to the repository root.
    pub(crate) dir: String,
    pub(crate) env: Vec<(String, String)>,
}

impl std::fmt::Display for Step {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (key, value) in &self.env {
            write!(f, "{key}={value} ")?;
        }
        write!(f, "{} {}", self.program, self.args.join(" "))
    }
}

pub(crate) fn step(program: &str, args: &[&str], dir: &str) -> Step {
    Step {
        program: program.to_string(),
        args: args.iter().map(|a| a.to_string()).collect(),
        dir: dir.to_string(),
        env: Vec::new(),
    }
}

pub(crate) fn run_steps(
    platform: &impl Platform,
    label: &str,
    steps: &[Step],
    root: &str,
) -> Result<(), CIError> {
    for step in steps {
        println!("[{label}] {step}");
        utilities::stream_command(
            platform,
            Command::new(&step.program)
                .args(&step.args)
                .envs(step.env.iter().map(|(k, v)| (k, v)))
                .current_dir(Path::new(root).join(&step.dir)),
        )?;
    }

    Ok(())
}

/// A named check (e.g. `lint`) of a project and the steps implementing it.
#[derive(Debug, PartialEq)]
pub(crate) struct Check {
    pub(crate) name: String,
    pub(crate) steps: Vec<Step>,
}

/// Runs the checks of the project until one fails. A check fails if it modifies or creates files
/// (e.g. outdated generated sources), otherwise it passes with a proof. A check proven with the
/// project's fingerprint and its steps is not run, each check builds what it needs itself (cargo,
/// turbo `dependsOn`). A passed check left the files as they were, so the fingerprint holds for
/// all. The setup of the project's kind (e.g. `pnpm install`) runs before the first check that runs.
pub(crate) fn check(
    platform: &impl Platform,
    project: &Project,
    projects: &[Project],
    root: &str,
    report: &mut Report,
    proofs: &Proofs,
) -> Result<(), CIError> {
    let before = Snapshot::of(platform, None)?;
    let fingerprint = affected::fingerprints(projects, &before)?
        .remove(&project.dir)
        .unwrap_or_default();
    let mut setup = Some(project.kind.setup()).filter(|steps| !steps.is_empty());
    let mut passed = true;

    for check in project.project_type.checks(project, projects, root) {
        if !passed {
            report.skip(&project.dir, Task::Check(&check.name));
            continue;
        }

        let key = proof::key(&project.dir, &check, &fingerprint);

        if proofs.proven(platform, &key) {
            println!("[{}] {}: proven ({key})", project.dir, check.name);
            report.proven(&project.dir, Task::Check(&check.name));
            continue;
        }

        if let Some(steps) = setup.take()
            && !report.run(&project.dir, Task::Setup, || {
                run_steps(platform, &project.dir, &steps, root)
            })
        {
            passed = false;
            report.skip(&project.dir, Task::Check(&check.name));
            continue;
        }

        passed = report.run(&project.dir, Task::Check(&check.name), || {
            run_steps(platform, &project.dir, &check.steps, root)?;
            let modified = modified(&before.files, &git::worktree_files(platform)?);

            if modified.is_empty() {
                Ok(())
            } else {
                Err(format!(
                    "'{}' of '{}' modified or created files, run it locally and commit the result:\n  {}",
                    check.name,
                    project.dir,
                    modified.join("\n  ")
                )
                .into())
            }
        });

        if passed {
            proofs.save(platform, &key)?;
        }
    }

    Ok(())
}

/// The files that differ between the two snapshots (path -> git object id).
fn modified(before: &BTreeMap<String, String>, after: &BTreeMap<String, String>) -> Vec<String> {
    let paths: BTreeSet<&String> = before.keys().chain(after.keys()).collect();
    paths
        .into_iter()
        .filter(|path| before.get(*path) != after.get(*path))
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modified_lists_changed_created_and_deleted_files() {
        let files = |files: &[(&str, &str)]| -> BTreeMap<String, String> {
            files
                .iter()
                .map(|(p, i)| (p.to_string(), i.to_string()))
                .collect()
        };
        let before = files(&[("a.rs", "1"), ("b.rs", "2"), ("d.rs", "4")]);
        let after = files(&[("a.rs", "1"), ("b.rs", "3"), ("c.rs", "5")]);

        assert_eq!(modified(&before, &after), vec!["b.rs", "c.rs", "d.rs"]);
        assert!(modified(&before, &before).is_empty());
    }
}
