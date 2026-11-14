mod affected;
mod bootstrap;
mod check;
mod ci_error;
mod config;
mod git;
mod language;
mod manifest;
mod project;
mod regenerate;
mod release;
mod report;
mod utilities;

use crate::affected::Job;
use crate::affected::Snapshot;
use crate::ci_error::CIError;
use crate::project::Project;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::path::Path;
use std::process::Command;

const USAGE: &str = "usage:
  agci                         update versions and regenerate the API clients
  agci projects                list the projects, their dependencies and checks
  agci affected [--base <ref>] JSON job matrix of the projects affected by the changes since <ref>
                               (merge base), or by the uncommitted changes without --base
  agci affected --all          JSON job matrix of all projects
  agci check [<project>...]    run the checks of the projects, default: the affected ones; all of
                               them run, failing ones are listed at the end
  agci config                  on GitHub Actions: the outputs of the config job for the event, the
                               check matrix of a pull request (without the jobs that already passed
                               in it) or the release of a push to main
  agci bootstrap [--check] <dir>
                               builds agci into <dir> (with the fingerprint of its sources); --check
                               only writes rebuild=<bool> (whether the agci in <dir> is outdated) to
                               the GitHub step outputs
  agci release (--base <ref>|--all|--projects [<project>...]) [--plan]
                               after a push to main: the release:dev hooks of the projects affected
                               since <ref> (or of the given ones) and, if there is no v<version> tag
                               for the root Cargo.toml version, the release (the packages and release
                               hooks of all projects, then the GitHub release and tag); --plan only
                               prints the steps";

fn root() -> Result<String, CIError> {
    Ok(
        utilities::run_quiet(Command::new("git").args(["rev-parse", "--show-toplevel"]))?
            .trim()
            .to_string(),
    )
}

fn load() -> Result<Vec<Project>, CIError> {
    let read = |path: &str| match std::fs::read_to_string(path) {
        Ok(content) => Ok(Some(content)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(CIError::from(format!("'{path}': {e}"))),
    };
    let mut file = match read(config::FILE)? {
        Some(content) => config::File::parse(&content)?,
        None => config::File::default(),
    };

    if let Some(content) = read("package.json")? {
        file.pnpm_checks = manifest::root_checks(&content)?;
    }

    project::projects(manifest::manifests()?, &file, |p| Path::new(p).exists())
}

fn env(name: &str) -> Result<String, CIError> {
    std::env::var(name).map_err(|_| format!("{name} not set").into())
}

fn head() -> Result<Snapshot, CIError> {
    Snapshot::of(Some("HEAD"))
}

/// Projects affected since the merge base with `base` (HEAD) or by the uncommitted changes.
fn affected_since<'a>(
    projects: &'a [Project],
    base: Option<&str>,
) -> Result<BTreeSet<&'a str>, CIError> {
    let (base, head) = match base {
        Some(base) => (Snapshot::of(Some(&git::merge_base(base)?))?, head()?),
        None => (head()?, Snapshot::of(None)?),
    };
    affected::affected(projects, &base, &head)
}

/// Whether the version of the package is already in the registry.
fn published(registry: release::Registry, name: &str, version: &str) -> Result<bool, CIError> {
    match registry {
        release::Registry::Crates => published_crate(name, version),
        release::Registry::Npm => published_npm(name, version),
    }
}

fn published_crate(name: &str, version: &str) -> Result<bool, CIError> {
    let status = utilities::run_quiet(Command::new("curl").args([
        "-s",
        "-o",
        "/dev/null",
        "-w",
        "%{http_code}",
        "-A",
        "agci (https://github.com/agnesoft/agdb)",
        &format!("https://crates.io/api/v1/crates/{name}/{version}"),
    ]))?;

    match status.trim() {
        "200" => Ok(true),
        "404" => Ok(false),
        s => Err(format!("crates.io responded {s} for {name}@{version}").into()),
    }
}

/// `npm view` prints nothing for an unknown version and fails with E404 for an unknown package.
fn published_npm(name: &str, version: &str) -> Result<bool, CIError> {
    let npm = if cfg!(windows) { "npm.cmd" } else { "npm" };
    let out = Command::new(npm)
        .args(["view", &format!("{name}@{version}"), "version"])
        .output()?;
    let stderr = String::from_utf8_lossy(&out.stderr);

    if out.status.success() {
        Ok(!out.stdout.trim_ascii().is_empty())
    } else if stderr.contains("E404") {
        Ok(false)
    } else {
        Err(format!("npm view {name}@{version} failed: {stderr}").into())
    }
}

fn release(args: &[String]) -> Result<(), CIError> {
    let (plan_only, args) = match args.split_last() {
        Some((last, rest)) if last == "--plan" => (true, rest),
        _ => (false, args),
    };

    if args.is_empty() {
        return Err(USAGE.into());
    }

    let projects = load()?;
    let affected_dirs: Vec<String> = match args {
        [flag, dirs @ ..] if flag == "--projects" => select(&projects, dirs)?
            .into_iter()
            .map(|p| p.dir.clone())
            .collect(),
        _ => affected(args)?.into_iter().map(|p| p.dir).collect(),
    };
    let steps = release_steps(&projects, &affected_dirs)?;

    if plan_only {
        for step in &steps {
            println!("{step}");
        }
        Ok(())
    } else {
        check::run_steps("release", &steps, &root()?)
    }
}

fn release_steps(projects: &[Project], affected: &[String]) -> Result<Vec<check::Step>, CIError> {
    let version = git::new_version()?;
    let release = release::Release {
        affected: affected.iter().map(String::as_str).collect(),
        version: (!git::tag_exists(&format!("v{version}"))?).then_some(version),
        sha: git::head()?,
    };
    release::plan(projects, &release, published)
}

/// The outputs of the `config` job from the GitHub event: the `matrix` of the projects affected by a
/// pull request (all of them for other events) and, for a push to main, no matrix but the affected
/// `projects` and `release=true` if there is something to release.
fn config() -> Result<(), CIError> {
    let projects = load()?;
    let event = env("GITHUB_EVENT_NAME")?;
    let dirs: BTreeSet<&str> = match event.as_str() {
        "pull_request" => affected_since(
            &projects,
            Some(&format!("origin/{}", env("GITHUB_BASE_REF")?)),
        )?,
        "push" => {
            let event: serde_json::Value =
                serde_json::from_str(&std::fs::read_to_string(env("GITHUB_EVENT_PATH")?)?)?;
            let before = event["before"].as_str().unwrap_or_default();
            let dirs: Vec<String> = if before.is_empty() || before.chars().all(|c| c == '0') {
                projects.iter().map(|p| p.dir.clone()).collect()
            } else {
                affected_since(&projects, Some(before))?
                    .into_iter()
                    .map(str::to_string)
                    .collect()
            };
            let steps = release_steps(&projects, &dirs)?;

            for step in &steps {
                println!("{step}");
            }

            utilities::set_output("matrix", "[]")?;
            utilities::set_output("projects", &dirs.join(" "))?;
            utilities::set_output("release", &(!steps.is_empty()).to_string())?;
            return Ok(());
        }
        _ => projects.iter().map(|p| p.dir.as_str()).collect(),
    };
    let mut jobs = affected::jobs(
        &projects,
        &dirs,
        &affected::fingerprints(&projects, &head()?)?,
    );

    if event == "pull_request" {
        skip_passed(&mut jobs);
    }

    println!("{}", serde_json::to_string_pretty(&jobs)?);
    utilities::set_output("matrix", &serde_json::to_string(&jobs)?)
}

/// Drops the jobs that already passed in this pull request with the same fingerprints (re-runs,
/// pushes not changing them): the check job saves its evidence to the actions cache. Without the
/// evidence (e.g. evicted) the job runs again.
fn skip_passed(jobs: &mut Vec<Job>) {
    match passed() {
        Ok(passed) => jobs.retain(|job| {
            let done = passed.contains(&job.evidence);

            if done {
                println!(
                    "{} ({:?}) already passed: {}",
                    job.name, job.os, job.evidence
                );
            }

            !done
        }),
        Err(e) => println!(
            "::warning::the evidence of passed jobs could not be read, checking all affected: {}",
            e.description
        ),
    }
}

/// The evidence (actions cache keys) saved by the pull request.
fn passed() -> Result<BTreeSet<String>, CIError> {
    let url = format!(
        "repos/{}/actions/caches?key={}&ref={}&per_page=100",
        env("GITHUB_REPOSITORY")?,
        affected::EVIDENCE,
        env("GITHUB_REF")?
    );
    Ok(utilities::run_quiet(Command::new("gh").args([
        "api",
        "--paginate",
        "--jq",
        ".actions_caches[].key",
        &url,
    ]))?
    .lines()
    .map(str::to_string)
    .collect())
}

fn select<'a>(projects: &'a [Project], dirs: &[String]) -> Result<Vec<&'a Project>, CIError> {
    dirs.iter()
        .map(|dir| {
            let dir = dir.trim_end_matches('/');
            projects
                .iter()
                .find(|p| p.dir == dir)
                .ok_or(CIError::from(format!("unknown project '{dir}'")))
        })
        .collect()
}

fn affected(args: &[String]) -> Result<Vec<Project>, CIError> {
    let projects = load()?;
    let dirs: Vec<String> = match args {
        [all] if all == "--all" => projects.iter().map(|p| p.dir.clone()).collect(),
        [base, r] if base == "--base" => affected_since(&projects, Some(r))?
            .into_iter()
            .map(str::to_string)
            .collect(),
        [] => affected_since(&projects, None)?
            .into_iter()
            .map(str::to_string)
            .collect(),
        _ => return Err(USAGE.into()),
    };
    Ok(projects
        .into_iter()
        .filter(|p| dirs.contains(&p.dir))
        .collect())
}

fn run(args: &[String]) -> Result<(), CIError> {
    std::env::set_current_dir(root()?)?;

    match args.first().map(String::as_str) {
        None => regenerate::regenerate(),
        Some("projects") => {
            for p in load()? {
                println!(
                    "{} ({}{}) -> [{}]",
                    p.dir,
                    p.kind,
                    if p.packages.is_empty() {
                        String::new()
                    } else {
                        format!(": {}", p.packages.join(", "))
                    },
                    p.depends.iter().cloned().collect::<Vec<_>>().join(", ")
                );
                let checks: Vec<&str> = p.config.checks.iter().map(|c| p.check_name(c)).collect();
                println!("  checks: {}", checks.join(", "));

                if !p.published.is_empty() {
                    let names: Vec<&str> = p.published.iter().map(|p| p.name.as_str()).collect();
                    println!("  publishes: {}", names.join(", "));
                }
            }
            Ok(())
        }
        Some("affected") => {
            let projects = affected(&args[1..])?;
            let all: BTreeSet<&str> = projects.iter().map(|p| p.dir.as_str()).collect();
            let fingerprints: BTreeMap<String, String> =
                affected::fingerprints(&load()?, &head()?)?;
            println!(
                "{}",
                serde_json::to_string(&affected::jobs(&projects, &all, &fingerprints))?
            );
            Ok(())
        }
        Some("check") => {
            let root = root()?;
            let projects = load()?;
            let selected: Vec<&Project> = if args.len() > 1 {
                select(&projects, &args[1..])?
            } else {
                let affected = affected_since(&projects, None)?;
                projects
                    .iter()
                    .filter(|p| affected.contains(p.dir.as_str()))
                    .collect()
            };

            let mut report = report::Report::new();

            for project in selected {
                check::check(project, &projects, &root, &mut report)?;
            }

            report.finish()
        }
        Some("config") if args.len() == 1 => config(),
        Some("bootstrap") => match &args[1..] {
            [flag, dir] if flag == "--check" => bootstrap::check(Path::new(dir), load()),
            [dir] => bootstrap::build(Path::new(dir), &load()?),
            _ => Err(USAGE.into()),
        },
        Some("release") => release(&args[1..]),
        Some(_) => Err(USAGE.into()),
    }
}

fn main() -> std::process::ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();

    match run(&args) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error: {}", e.description);
            std::process::ExitCode::FAILURE
        }
    }
}
