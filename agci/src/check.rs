use crate::CIError;
use crate::config::RUST_CHECKS;
use crate::manifest::Lang;
use crate::project::Project;
use crate::utilities;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::path::Path;
use std::process::Command;

#[cfg(target_os = "windows")]
pub(crate) const PNPM: &str = "pnpm.cmd";
#[cfg(not(target_os = "windows"))]
pub(crate) const PNPM: &str = "pnpm";

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

pub(crate) fn run_steps(label: &str, steps: &[Step], root: &str) -> Result<(), CIError> {
    for step in steps {
        println!("[{label}] {step}");
        utilities::run_command(
            Command::new(&step.program)
                .args(&step.args)
                .envs(step.env.iter().map(|(k, v)| (k, v)))
                .current_dir(Path::new(root).join(&step.dir)),
        )?;
    }

    Ok(())
}

/// Runs the checks of the project. Fails if they modify tracked files, e.g. outdated generated sources.
pub(crate) fn check(project: &Project, projects: &[Project], root: &str) -> Result<(), CIError> {
    let before = tracked_changes()?;
    run_steps(&project.dir, &steps(project, projects, root), root)?;
    let after = tracked_changes()?;
    let modified = modified_files(&before, &after);

    if !modified.is_empty() {
        return Err(format!(
            "checks of '{}' modified tracked files, run them locally and commit the result:\n  {}",
            project.dir,
            modified.join("\n  ")
        )
        .into());
    }

    Ok(())
}

fn tracked_changes() -> Result<String, CIError> {
    utilities::run_quiet(Command::new("git").args(["diff", "--no-renames", "HEAD"]))
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

pub(crate) fn steps(project: &Project, projects: &[Project], root: &str) -> Vec<Step> {
    match project.lang {
        Lang::Rust => rust_steps(project, projects, root),
        Lang::Node => node_steps(project),
    }
}

pub(crate) fn pnpm_install() -> Step {
    step(PNPM, &["install", "--frozen-lockfile"], "")
}

fn node_steps(project: &Project) -> Vec<Step> {
    let mut steps = vec![pnpm_install()];
    steps.extend(node_scripts(project, &project.config.checks));
    steps
}

/// The scripts run via turbo over all packages of the project (in the packages that define them).
pub(crate) fn node_scripts(project: &Project, scripts: &[String]) -> Vec<Step> {
    let filters: Vec<String> = project
        .packages
        .iter()
        .map(|p| format!("--filter={p}"))
        .collect();
    let mut steps = Vec::new();

    for script in scripts {
        let mut args = vec![
            "exec",
            "turbo",
            "run",
            script,
            "--force",
            "--env-mode=loose",
        ];
        args.extend(filters.iter().map(String::as_str));
        steps.push(step(PNPM, &args, ""));
    }

    steps
}

fn rust_steps(project: &Project, projects: &[Project], root: &str) -> Vec<Step> {
    let config = &project.config;
    let name = project.packages[0].as_str();
    let dir = project.dir.as_str();
    let features: Vec<&str> = match &config.features {
        None => vec!["--all-features"],
        Some(f) if f.is_empty() => vec![],
        Some(f) => vec!["--features", f],
    };
    let cargo = |args: &[&str], extra: &[&str]| {
        let mut all = args.to_vec();
        all.extend(extra);
        step("cargo", &all, "")
    };
    let mut steps = Vec::new();

    for check in RUST_CHECKS
        .iter()
        .filter(|c| config.checks.iter().any(|d| d == *c))
    {
        match *check {
            "format" => steps.push(cargo(&["fmt", "-p", name, "--check"], &[])),
            "lint" => {
                steps.push(cargo(
                    &[&["clippy", "-p", name, "--all-targets"], &features[..]].concat(),
                    &["--", "-D", "warnings"],
                ));

                for combination in &config.feature_combinations {
                    for targets in ["--lib", "--all-targets"] {
                        let mut args = vec!["clippy", "-p", name, targets, "--no-default-features"];

                        if !combination.is_empty() {
                            args.extend(["--features", combination]);
                        }

                        steps.push(cargo(&args, &["--", "-D", "warnings"]));
                    }
                }
            }
            "testcheck" => steps.push(step("cargo", &["run", "-r", "-p", "agtestcheck"], dir)),
            "doc" => steps.push(cargo(
                &[&["test", "-p", name], &features[..]].concat(),
                &["--doc"],
            )),
            "bench" => steps.push(cargo(
                &[&["bench", "-p", name], &features[..]].concat(),
                &["--", "--samples", "1", "--iters", "1"],
            )),
            "test" => match config.coverage {
                None => steps.push(cargo(&[&["test", "-p", name], &features[..]].concat(), &[])),
                Some(coverage) => {
                    let ignore = coverage_ignore(project, projects, root);
                    let threshold = coverage.to_string();
                    let thresholds: Vec<&str> = match coverage {
                        100 => vec![
                            "--fail-uncovered-lines",
                            "0",
                            "--fail-uncovered-regions",
                            "0",
                            "--fail-uncovered-functions",
                            "0",
                        ],
                        0 => vec![],
                        _ => vec!["--fail-under-lines", &threshold],
                    };
                    steps.push(cargo(&["llvm-cov", "clean", "--workspace"], &[]));
                    steps.push(cargo(
                        &[&["llvm-cov", "-p", name], &features[..]].concat(),
                        &[
                            &["--show-missing-lines", "--ignore-filename-regex", &ignore],
                            &thresholds[..],
                        ]
                        .concat(),
                    ));
                }
            },
            _ => unreachable!("validated by project"),
        }
    }

    for command in &config.run {
        steps.push(step(utilities::BASH, &["-c", command], dir));
    }

    steps
}

/// Coverage counts only the project's own sources: other Rust projects and its examples are ignored.
fn coverage_ignore(project: &Project, projects: &[Project], root: &str) -> String {
    let sep = r"[/\\]";
    let path = |p: &str| p.split('/').map(regex_escape).collect::<Vec<_>>().join(sep);
    let mut ignored: Vec<String> = projects
        .iter()
        .filter(|p| {
            p.lang == Lang::Rust
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
mod tests {
    use super::*;
    use crate::manifest::Dep;
    use crate::project::projects;
    use crate::project::tests::config;
    use crate::project::tests::manifest;

    fn command_lines(project: &str, edit: impl FnOnce(&mut crate::config::Config)) -> Vec<String> {
        let mut projects = projects(
            vec![
                manifest("agdb", Lang::Rust, vec![], Some(config(&["test"], &[]))),
                manifest(
                    "agdb_server",
                    Lang::Rust,
                    vec![Dep::Dir("agdb".to_string())],
                    Some(config(&["test"], &[])),
                ),
                manifest(
                    "ui",
                    Lang::Node,
                    vec![],
                    Some(config(&["build", "test"], &[])),
                ),
                manifest("ui/lib", Lang::Node, vec![], None),
            ],
            |_| false,
        )
        .unwrap_or_else(|_| unreachable!());
        let index = projects.iter().position(|p| p.dir == project).unwrap();
        edit(&mut projects[index].config);

        steps(&projects[index], &projects, "/repo")
            .into_iter()
            .map(|s| format!("{}> {} {}", s.dir, s.program, s.args.join(" ")))
            .collect()
    }

    #[test]
    fn node_runs_scripts_in_declared_order_over_all_packages() {
        assert_eq!(
            command_lines("ui", |c| c.checks =
                vec!["test".to_string(), "build".to_string()]),
            vec![
                format!("> {PNPM} install --frozen-lockfile"),
                format!(
                    "> {PNPM} exec turbo run test --force --env-mode=loose --filter=ui --filter=ui_lib"
                ),
                format!(
                    "> {PNPM} exec turbo run build --force --env-mode=loose --filter=ui --filter=ui_lib"
                ),
            ]
        );
    }

    #[test]
    fn rust_runs_checks_in_canonical_order() {
        assert_eq!(
            command_lines("agdb_server", |c| {
                c.checks = ["test", "doc", "lint", "format"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect();
                c.features = Some("tls".to_string());
                c.run = vec!["./bench.sh".to_string()];
            }),
            vec![
                "> cargo fmt -p agdb_server --check",
                "> cargo clippy -p agdb_server --all-targets --features tls -- -D warnings",
                "> cargo test -p agdb_server --features tls --doc",
                "> cargo test -p agdb_server --features tls",
                &format!("agdb_server> {} -c ./bench.sh", utilities::BASH),
            ]
        );
    }

    #[test]
    fn rust_feature_combinations() {
        assert_eq!(
            command_lines("agdb", |c| {
                c.checks = vec!["lint".to_string()];
                c.features = Some(String::new());
                c.feature_combinations = vec![String::new(), "fs,env".to_string()];
            }),
            vec![
                "> cargo clippy -p agdb --all-targets -- -D warnings",
                "> cargo clippy -p agdb --lib --no-default-features -- -D warnings",
                "> cargo clippy -p agdb --all-targets --no-default-features -- -D warnings",
                "> cargo clippy -p agdb --lib --no-default-features --features fs,env -- -D warnings",
                "> cargo clippy -p agdb --all-targets --no-default-features --features fs,env -- -D warnings",
            ]
        );
    }

    #[test]
    fn rust_full_coverage_ignores_other_projects() {
        let sep = r"[/\\]";

        assert_eq!(
            command_lines("agdb", |c| c.coverage = Some(100)),
            vec![
                "> cargo llvm-cov clean --workspace".to_string(),
                format!(
                    "> cargo llvm-cov -p agdb --all-features --show-missing-lines --ignore-filename-regex ^{sep}repo{sep}(agdb_server|agdb{sep}examples){sep} --fail-uncovered-lines 0 --fail-uncovered-regions 0 --fail-uncovered-functions 0"
                ),
            ]
        );
    }

    #[test]
    fn rust_partial_coverage_threshold() {
        let lines = command_lines("agdb", |c| c.coverage = Some(90));

        assert!(lines[1].ends_with(" --fail-under-lines 90"), "{}", lines[1]);
        assert!(!command_lines("agdb", |c| c.coverage = Some(0))[1].contains("--fail"));
    }

    #[test]
    fn modified_files_compares_per_file_diffs() {
        let before = "diff --git a/a.rs b/a.rs\n-x\n+y\ndiff --git a/b.rs b/b.rs\n-1\n+2\n";
        let after = "diff --git a/a.rs b/a.rs\n-x\n+y\ndiff --git a/b.rs b/b.rs\n-1\n+3\ndiff --git a/c.rs b/c.rs\n+z\n";

        assert_eq!(modified_files(before, after), vec!["b.rs", "c.rs"]);
        assert!(modified_files(before, before).is_empty());
    }

    #[test]
    fn regex_escape_special_characters() {
        assert_eq!(regex_escape("a.b-c(d)"), r"a\.b-c\(d\)");
    }
}
