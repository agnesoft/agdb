use crate::CIError;
use crate::config::CARGO_CHECKS;
use crate::manifest::Kind;
use crate::project::Project;
use crate::report::Report;
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
        utilities::stream_command(
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

fn named(name: &str, steps: Vec<Step>) -> Check {
    Check {
        name: name.to_string(),
        steps,
    }
}

const UNCHANGED: &str = "unchanged files";

/// Runs the checks of the project until one fails, then whether they left the files unchanged
/// (e.g. outdated generated sources).
pub(crate) fn check(
    project: &Project,
    projects: &[Project],
    root: &str,
    report: &mut Report,
) -> Result<(), CIError> {
    let before = tracked_changes()?;
    let untracked_before = untracked()?;
    let mut passed = true;

    for check in checks(project, projects, root) {
        if passed {
            passed = report.run(&project.dir, &check.name, || {
                run_steps(&project.dir, &check.steps, root)
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
        let mut modified = modified_files(&before, &tracked_changes()?);
        modified.extend(untracked()?.difference(&untracked_before).cloned());

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

fn tracked_changes() -> Result<String, CIError> {
    utilities::run_quiet(Command::new("git").args(["diff", "--no-renames", "HEAD"]))
}

/// Untracked files that are not ignored, e.g. newly generated sources.
fn untracked() -> Result<BTreeSet<String>, CIError> {
    Ok(utilities::run_quiet(Command::new("git").args([
        "ls-files",
        "--others",
        "--exclude-standard",
    ]))?
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

pub(crate) fn checks(project: &Project, projects: &[Project], root: &str) -> Vec<Check> {
    match project.kind {
        Kind::Cargo => cargo_checks(project, projects, root),
        Kind::Pnpm => pnpm_checks(project),
        Kind::Shell => project
            .config
            .checks
            .iter()
            .zip(shell_scripts(project, &project.config.checks))
            .map(|(name, step)| named(name, vec![step]))
            .collect(),
    }
}

/// The Shell scripts of the project run in its directory, `.sh` by bash and `.ps1` by PowerShell.
pub(crate) fn shell_scripts(project: &Project, names: &[String]) -> Vec<Step> {
    names
        .iter()
        .filter_map(|name| project.shell_script(name))
        .map(|file| {
            if file.ends_with(".ps1") {
                step("pwsh", &["-NoProfile", "-File", file], &project.dir)
            } else {
                step(utilities::BASH, &[file], &project.dir)
            }
        })
        .collect()
}

pub(crate) fn pnpm_install() -> Step {
    step(PNPM, &["install", "--frozen-lockfile"], "")
}

fn pnpm_checks(project: &Project) -> Vec<Check> {
    let mut checks = vec![named("install", vec![pnpm_install()])];
    checks.extend(
        project
            .config
            .checks
            .iter()
            .zip(pnpm_scripts(project, &project.config.checks))
            .map(|(script, step)| named(script, vec![step])),
    );
    checks
}

/// The scripts run via turbo over all packages of the project (in the packages that define them).
pub(crate) fn pnpm_scripts(project: &Project, scripts: &[String]) -> Vec<Step> {
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
            // the default (`grouped` on CI) opens its own GitHub log groups, which do not nest in agci's
            "--log-order=stream",
        ];
        args.extend(filters.iter().map(String::as_str));
        steps.push(step(PNPM, &args, ""));
    }

    steps
}

fn cargo_checks(project: &Project, projects: &[Project], root: &str) -> Vec<Check> {
    let config = &project.config;
    let name = project.packages[0].as_str();
    let dir = project.dir.as_str();
    let features = ["--all-features"];
    let cargo = |args: &[&str], extra: &[&str]| {
        let mut all = args.to_vec();
        all.extend(extra);
        step("cargo", &all, "")
    };
    let mut checks = Vec::new();

    for check in CARGO_CHECKS
        .iter()
        .filter(|c| config.checks.iter().any(|d| d == *c))
    {
        let mut steps = Vec::new();

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
                &[&["bench", "-p", name, "--bench", "*"], &features[..]].concat(),
                &["--", "--samples", "1", "--iters", "1"],
            )),
            "coverage" if !config.coverage() => {
                steps.push(cargo(&[&["test", "-p", name], &features[..]].concat(), &[]));
            }
            "coverage" => {
                let ignore = coverage_ignore(project, projects, root);
                steps.push(cargo(&["llvm-cov", "clean", "--workspace"], &[]));
                steps.push(cargo(
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
                ));
            }
            _ => unreachable!("validated by project"),
        }

        checks.push(named(project.check_name(check), steps));
    }

    for check in config
        .checks
        .iter()
        .filter(|c| !CARGO_CHECKS.contains(&c.as_str()))
    {
        checks.push(named(
            check,
            shell_scripts(project, std::slice::from_ref(check)),
        ));
    }

    checks
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
mod tests {
    use super::*;
    use crate::manifest::Dep;
    use crate::project::tests::build;
    use crate::project::tests::config;
    use crate::project::tests::manifest;

    fn command_lines(project: &str, edit: impl FnOnce(&mut crate::config::Config)) -> Vec<String> {
        let mut projects = build(
            vec![
                manifest(
                    "agdb",
                    Kind::Cargo,
                    vec![],
                    Some(config(&["coverage"], &[])),
                ),
                manifest(
                    "agdb_server",
                    Kind::Cargo,
                    vec![Dep::Dir("agdb".to_string())],
                    Some(config(&["coverage", "smoke"], &[])),
                ),
                manifest(
                    "ui",
                    Kind::Pnpm,
                    vec![],
                    Some(config(&["build", "test"], &[])),
                ),
                manifest("ui/lib", Kind::Pnpm, vec![], None),
                manifest(
                    "agdb_server/image",
                    Kind::Shell,
                    vec![],
                    Some(config(&["test", "test:functional", "smoke"], &[])),
                ),
            ],
            |p| {
                [
                    "agdb_server/smoke.sh",
                    "agdb_server/image",
                    "agdb_server/image/test.sh",
                    "agdb_server/image/test_functional.ps1",
                    "agdb_server/image/smoke.sh",
                    "agdb_server/image/smoke.ps1",
                ]
                .contains(&p)
            },
        )
        .unwrap_or_else(|_| unreachable!());
        let index = projects.iter().position(|p| p.dir == project).unwrap();
        edit(&mut projects[index].config);

        checks(&projects[index], &projects, "/repo")
            .into_iter()
            .flat_map(|c| c.steps)
            .map(|s| format!("{}> {} {}", s.dir, s.program, s.args.join(" ")))
            .collect()
    }

    fn check_names(project: &str, edit: impl FnOnce(&mut crate::config::Config)) -> Vec<String> {
        let projects = build(
            vec![
                manifest(
                    "agdb",
                    Kind::Cargo,
                    vec![],
                    Some(config(&["coverage", "regenerate"], &[])),
                ),
                manifest("ui", Kind::Pnpm, vec![], Some(config(&["test"], &[]))),
            ],
            |p| p == "agdb/regenerate.ps1",
        )
        .unwrap_or_else(|_| unreachable!());
        let mut project = projects.into_iter().find(|p| p.dir == project).unwrap();
        edit(&mut project.config);
        checks(&project, &[], "/repo")
            .into_iter()
            .map(|c| c.name)
            .collect()
    }

    #[test]
    fn checks_are_named_after_the_metadata() {
        assert_eq!(
            check_names("agdb", |c| {
                c.checks = ["regenerate", "coverage", "lint"]
                    .map(str::to_string)
                    .to_vec();
            }),
            vec!["lint", "coverage", "regenerate"]
        );
        assert_eq!(
            check_names("agdb", |c| {
                c.checks = vec!["coverage".to_string()];
                c.coverage = Some(false);
            }),
            vec!["test"]
        );
        assert_eq!(
            check_names("ui", |c| c.checks =
                vec!["test".to_string(), "build".to_string()]),
            vec!["install", "test", "build"]
        );
    }

    #[test]
    fn node_runs_scripts_in_declared_order_over_all_packages() {
        assert_eq!(
            command_lines("ui", |c| c.checks =
                vec!["test".to_string(), "build".to_string()]),
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

    #[test]
    fn shell_runs_script_files_in_the_project_directory() {
        let smoke = if cfg!(windows) {
            "agdb_server/image> pwsh -NoProfile -File smoke.ps1".to_string()
        } else {
            format!("agdb_server/image> {} smoke.sh", utilities::BASH)
        };

        assert_eq!(
            command_lines("agdb_server/image", |_| {}),
            vec![
                format!("agdb_server/image> {} test.sh", utilities::BASH),
                "agdb_server/image> pwsh -NoProfile -File test_functional.ps1".to_string(),
                smoke,
            ]
        );
    }

    #[test]
    fn rust_runs_checks_in_canonical_order() {
        assert_eq!(
            command_lines("agdb_server", |c| {
                c.checks = ["smoke", "coverage", "bench", "doc", "lint", "format"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect();
                c.coverage = Some(false);
            }),
            vec![
                "> cargo fmt -p agdb_server --check",
                "> cargo clippy -p agdb_server --all-targets --all-features -- -D warnings",
                "> cargo test -p agdb_server --all-features --doc",
                "> cargo bench -p agdb_server --bench * --all-features -- --samples 1 --iters 1",
                "> cargo test -p agdb_server --all-features",
                &format!("agdb_server> {} smoke.sh", utilities::BASH),
            ]
        );
    }

    #[test]
    fn rust_feature_combinations() {
        assert_eq!(
            command_lines("agdb", |c| {
                c.checks = vec!["lint".to_string()];
                c.feature_combinations = vec![String::new(), "fs,env".to_string()];
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
    fn rust_full_coverage_ignores_other_projects() {
        let sep = r"[/\\]";

        assert_eq!(
            command_lines("agdb", |c| c.checks = vec!["coverage".to_string()]),
            vec![
                "> cargo llvm-cov clean --workspace".to_string(),
                format!(
                    "> cargo llvm-cov -p agdb --all-features --show-missing-lines --ignore-filename-regex ^{sep}repo{sep}(agdb_server|agdb{sep}examples){sep} --fail-uncovered-lines 0 --fail-uncovered-regions 0 --fail-uncovered-functions 0"
                ),
            ]
        );
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
