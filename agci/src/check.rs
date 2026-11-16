use crate::CIError;
use crate::project::Project;
use crate::report::Report;
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

const UNCHANGED: &str = "unchanged files";

/// Runs the checks of the project until one fails, then whether they left the files unchanged
/// (e.g. outdated generated sources).
pub(crate) fn check(
    platform: &impl Platform,
    project: &Project,
    projects: &[Project],
    root: &str,
    report: &mut Report,
) -> Result<(), CIError> {
    let before = tracked_changes(platform)?;
    let untracked_before = untracked(platform)?;
    let mut passed = true;

    for check in project.project_type.checks(project, projects, root) {
        if passed {
            passed = report.run(&project.dir, &check.name, || {
                run_steps(platform, &project.dir, &check.steps, root)
            });
        } else {
            report.skip(&project.dir, &check.name);
        }
    }

    if !passed {
        report.skip(&project.dir, UNCHANGED);
        return Ok(());
    }

    report.run(&project.dir, UNCHANGED, || {
        let mut modified = modified_files(&before, &tracked_changes(platform)?);
        modified.extend(untracked(platform)?.difference(&untracked_before).cloned());

        if modified.is_empty() {
            Ok(())
        } else {
            Err(format!(
                "checks of '{}' modified or created files, run them locally and commit the result:\n  {}",
                project.dir,
                modified.join("\n  ")
            )
            .into())
        }
    });

    Ok(())
}

fn tracked_changes(platform: &impl Platform) -> Result<String, CIError> {
    utilities::run_quiet(
        platform,
        Command::new("git").args(["diff", "--no-renames", "HEAD"]),
    )
}

/// Untracked files that are not ignored, e.g. newly generated sources.
fn untracked(platform: &impl Platform) -> Result<BTreeSet<String>, CIError> {
    Ok(utilities::run_quiet(
        platform,
        Command::new("git").args(["ls-files", "--others", "--exclude-standard"]),
    )?
    .lines()
    .map(str::to_string)
    .collect())
}

/// Files whose diff against HEAD differs between the two `git diff HEAD` outputs.
fn modified_files(before: &str, after: &str) -> Vec<String> {
    fn per_file(diff: &str) -> BTreeMap<&str, &str> {
        diff.split("diff --git a/")
            .skip(1)
            .filter_map(|chunk| chunk.split_once(' ').map(|(file, _)| (file, chunk)))
            .collect()
    }

    let before = per_file(before);
    let after = per_file(after);
    let files: BTreeSet<&str> = before.keys().chain(after.keys()).copied().collect();
    files
        .into_iter()
        .filter(|f| before.get(f) != after.get(f))
        .map(str::to_string)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modified_files_compares_per_file_diffs() {
        let before = "diff --git a/a.rs b/a.rs\n-x\n+y\ndiff --git a/b.rs b/b.rs\n-1\n+2\n";
        let after = "diff --git a/a.rs b/a.rs\n-x\n+y\ndiff --git a/b.rs b/b.rs\n-1\n+3\ndiff --git a/c.rs b/c.rs\n+z\n";

        assert_eq!(modified_files(before, after), vec!["b.rs", "c.rs"]);
        assert!(modified_files(before, before).is_empty());
    }
}
