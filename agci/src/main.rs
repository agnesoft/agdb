mod affected;
mod bootstrap;
mod check;
mod ci_error;
mod config;
mod generate;
mod git;
mod manifest;
mod project;
mod project_type;
mod proof;
mod regenerate;
mod release;
mod report;
mod roundtrip;
mod utilities;

use crate::affected::Job;
use crate::affected::Snapshot;
use crate::ci_error::CIError;
use crate::config::Os;
use crate::project::Project;
use agplatform::Env;
use agplatform::Fs;
use agplatform::Platform;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::path::Path;
use std::process::Command;
use std::time::SystemTime;

const USAGE: &str = "usage:
  agci                         update the versions in the manifests to the repository version and
                               run the regenerate hooks of the projects
  agci projects                list the projects, their dependencies and checks
  agci affected [--base <ref>] JSON job matrix of the projects affected by the changes since <ref>
                               (merge base), or by the uncommitted changes without --base
  agci affected --all          JSON job matrix of all projects
  agci check [--force] [-p <project>]...
                               run the checks of the given projects, default: those affected by the
                               uncommitted changes or, in a clean worktree, since origin/main
                               (fetched, merge base) that run on this OS; all of them run, failing
                               ones are listed at the end; a check that passed before with the
                               same fingerprint and steps is skipped unless --force
  agci check --clean           remove all proofs of passed checks
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
                               for the repository version, the release (the packages and release
                               hooks of all projects, then the GitHub release and tag); --plan only
                               prints the steps
  agci generate [--dump <dir>] the query builder of the clients from the agdb source, with the
                               lowered module after each pass in <dir> (default
                               target/agci/generate)
  agci roundtrip <package>...  check that every item of the packages' libraries and tests, printed
                               from its agsource IR, is the item as written";

/// What `check` compares a clean worktree to: `<REMOTE>/<BRANCH>`.
const REMOTE: &str = "origin";
const BRANCH: &str = "main";

fn root(platform: &impl Platform) -> Result<String, CIError> {
    Ok(utilities::run_quiet(
        platform,
        Command::new("git").args(["rev-parse", "--show-toplevel"]),
    )?
    .trim()
    .to_string())
}

/// `agci.toml`, the defaults without it.
fn config_file(platform: &impl Platform) -> Result<config::File, CIError> {
    match utilities::read_if_exists(platform, config::FILE)? {
        Some(content) => config::File::parse(&content),
        None => Ok(config::File::default()),
    }
}

fn load(platform: &impl Platform) -> Result<Vec<Project>, CIError> {
    let read = |path: &str| utilities::read_if_exists(platform, path);
    let mut file = config_file(platform)?;

    for kind in project_type::Kind::ALL {
        if let Some(checks) = kind.root_checks(&read)? {
            file.root_checks.insert(kind, checks);
        }
    }

    project::projects(manifest::manifests(platform)?, &file, |p| {
        platform.fs().exists(p).unwrap_or(false)
    })
}

fn env(platform: &impl Platform, name: &str) -> Result<String, CIError> {
    platform
        .env()
        .var(name)
        .and_then(|value| value.into_string().ok())
        .ok_or_else(|| format!("{name} not set").into())
}

fn head(platform: &impl Platform) -> Result<Snapshot, CIError> {
    Snapshot::of(platform, Some("HEAD"))
}

/// Projects affected since the merge base with `base` (HEAD) or by the uncommitted changes.
fn affected_since<'a>(
    platform: &impl Platform,
    projects: &'a [Project],
    base: Option<&str>,
) -> Result<BTreeSet<&'a str>, CIError> {
    let (base, head) = match base {
        Some(base) => (
            Snapshot::of(platform, Some(&git::merge_base(platform, base)?))?,
            head(platform)?,
        ),
        None => (head(platform)?, Snapshot::of(platform, None)?),
    };
    affected::affected(projects, &base, &head)
}

/// The projects `check` runs without `-p`: those affected by the uncommitted changes or, in a clean
/// worktree, by the commits since the merge base with the fetched [`REMOTE`]/[`BRANCH`], if they run
/// on this OS. Says so if there are none, as the report is empty then.
fn to_check<'a>(
    platform: &impl Platform,
    projects: &'a [Project],
) -> Result<Vec<&'a Project>, CIError> {
    let base = format!("{REMOTE}/{BRANCH}");
    let base = git::uncommitted(platform)?
        .is_empty()
        .then_some(base.as_str());

    if base.is_some()
        && let Err(e) = git::fetch(platform, REMOTE, BRANCH)
    {
        println!(
            "Warning: fetching {REMOTE} {BRANCH} failed, using the local {REMOTE}/{BRANCH}: {}",
            e.description
        );
    }

    let affected = affected_since(platform, projects, base)?;
    let (selected, _) = affected::on_os(projects, &affected, Os::CURRENT);

    if selected.is_empty() {
        match base {
            Some(base) => println!("Nothing affected since {base} (clean worktree)"),
            None => println!("Nothing affected by the uncommitted changes"),
        }
    }

    Ok(selected)
}

#[derive(Debug, PartialEq)]
enum CheckArgs {
    /// `--clean`: remove all proofs.
    Clean,
    /// `[--force] [-p <project>]...`: whether to rerun proven checks and the given projects.
    Run { force: bool, dirs: Vec<String> },
}

fn check_args(args: &[String]) -> Result<CheckArgs, CIError> {
    if let [clean] = args
        && clean == "--clean"
    {
        return Ok(CheckArgs::Clean);
    }

    let mut force = false;
    let mut dirs = Vec::new();
    let mut args = args.iter();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--force" => force = true,
            "-p" => dirs.push(args.next().ok_or(USAGE)?.clone()),
            _ => return Err(USAGE.into()),
        }
    }

    Ok(CheckArgs::Run { force, dirs })
}

/// Checks the projects, skipping the checks proven with their current fingerprint, after removing
/// the proofs not used for the `[proofs]` `max_age_days`.
fn check(platform: &impl Platform, args: &[String]) -> Result<(), CIError> {
    let (force, dirs) = match check_args(args)? {
        CheckArgs::Clean => {
            return proof::Proofs::new(platform, false)?.clean(platform);
        }
        CheckArgs::Run { force, dirs } => (force, dirs),
    };
    let root = root(platform)?;
    let projects = load(platform)?;
    let selected = if dirs.is_empty() {
        to_check(platform, &projects)?
    } else {
        select(&projects, &dirs)?
    };
    let proofs = proof::Proofs::new(platform, force)?;
    proofs.prune(
        platform,
        SystemTime::now(),
        config_file(platform)?.proofs.max_age(),
    )?;
    let mut report = report::Report::new(platform);

    for project in selected {
        check::check(platform, project, &projects, &root, &mut report, &proofs)?;
    }

    report.finish(platform)
}

fn release(platform: &impl Platform, args: &[String]) -> Result<(), CIError> {
    let (plan_only, args) = match args.split_last() {
        Some((last, rest)) if last == "--plan" => (true, rest),
        _ => (false, args),
    };

    if args.is_empty() {
        return Err(USAGE.into());
    }

    let projects = load(platform)?;
    let affected_dirs: Vec<String> = match args {
        [flag, dirs @ ..] if flag == "--projects" => select(&projects, dirs)?
            .into_iter()
            .map(|p| p.dir.clone())
            .collect(),
        _ => affected(platform, args)?
            .into_iter()
            .map(|p| p.dir)
            .collect(),
    };
    let steps = release_steps(platform, &projects, &affected_dirs)?;

    if plan_only {
        for step in &steps {
            println!("{step}");
        }
        Ok(())
    } else {
        check::run_steps(
            platform,
            "release",
            &steps,
            &root(platform)?,
            &mut std::io::stdout(),
        )
    }
}

fn release_steps(
    platform: &impl Platform,
    projects: &[Project],
    affected: &[String],
) -> Result<Vec<check::Step>, CIError> {
    let version = project_type::version(platform)?;
    let release = release::Release {
        affected: affected.iter().map(String::as_str).collect(),
        version: (!git::tag_exists(platform, &format!("v{version}"))?).then_some(version),
        sha: git::head(platform)?,
    };
    release::plan(projects, &release, |registry, name, version| {
        registry.published(platform, name, version).map_err(|e| {
            format!(
                "{} lookup of {name}@{version} failed: {}",
                registry.name(),
                e.description
            )
            .into()
        })
    })
}

/// The outputs of the `config` job from the GitHub event: the `matrix` of the projects affected by a
/// pull request (all of them for other events) and, for a push to main, no matrix but the affected
/// `projects` and `release=true` if there is something to release.
fn config(platform: &impl Platform) -> Result<(), CIError> {
    let projects = load(platform)?;
    let event = env(platform, "GITHUB_EVENT_NAME")?;
    let dirs: BTreeSet<&str> = match event.as_str() {
        "pull_request" => affected_since(
            platform,
            &projects,
            Some(&format!("origin/{}", env(platform, "GITHUB_BASE_REF")?)),
        )?,
        "push" => {
            let event: serde_json::Value = serde_json::from_str(
                &platform
                    .fs()
                    .read_to_string(env(platform, "GITHUB_EVENT_PATH")?)?,
            )?;
            let before = event["before"].as_str().unwrap_or_default();
            let dirs: Vec<String> = if before.is_empty() || before.chars().all(|c| c == '0') {
                projects.iter().map(|p| p.dir.clone()).collect()
            } else {
                affected_since(platform, &projects, Some(before))?
                    .into_iter()
                    .map(str::to_string)
                    .collect()
            };
            let steps = release_steps(platform, &projects, &dirs)?;

            for step in &steps {
                println!("{step}");
            }

            utilities::set_output(platform, "matrix", "[]")?;
            utilities::set_output(platform, "projects", &dirs.join(" "))?;
            utilities::set_output(platform, "release", &(!steps.is_empty()).to_string())?;
            return Ok(());
        }
        _ => projects.iter().map(|p| p.dir.as_str()).collect(),
    };
    let mut jobs = affected::jobs(
        &projects,
        &dirs,
        &affected::fingerprints(&projects, &head(platform)?)?,
    );

    if event == "pull_request" {
        skip_passed(platform, &mut jobs);
    }

    println!("{}", serde_json::to_string_pretty(&jobs)?);
    utilities::set_output(platform, "matrix", &serde_json::to_string(&jobs)?)
}

/// Drops the jobs that already passed in this pull request with the same fingerprints (re-runs,
/// pushes not changing them): the check job saves its evidence to the actions cache. Without the
/// evidence (e.g. evicted) the job runs again.
fn skip_passed(platform: &impl Platform, jobs: &mut Vec<Job>) {
    match passed(platform) {
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
fn passed(platform: &impl Platform) -> Result<BTreeSet<String>, CIError> {
    let url = format!(
        "repos/{}/actions/caches?key={}&ref={}&per_page=100",
        env(platform, "GITHUB_REPOSITORY")?,
        affected::EVIDENCE,
        env(platform, "GITHUB_REF")?
    );
    Ok(utilities::run_quiet(
        platform,
        Command::new("gh").args(["api", "--paginate", "--jq", ".actions_caches[].key", &url]),
    )?
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

fn affected(platform: &impl Platform, args: &[String]) -> Result<Vec<Project>, CIError> {
    let projects = load(platform)?;
    let dirs: Vec<String> = match args {
        [all] if all == "--all" => projects.iter().map(|p| p.dir.clone()).collect(),
        [base, r] if base == "--base" => affected_since(platform, &projects, Some(r))?
            .into_iter()
            .map(str::to_string)
            .collect(),
        [] => affected_since(platform, &projects, None)?
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

fn run(platform: &impl Platform, args: &[String]) -> Result<(), CIError> {
    platform.env().set_current_dir(root(platform)?)?;

    match args.first().map(String::as_str) {
        None => regenerate::regenerate(platform, &load(platform)?, &root(platform)?),
        Some("projects") => {
            for p in load(platform)? {
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
                println!("  checks: {}", p.project_type.check_names().join(", "));

                if !p.published.is_empty() {
                    let names: Vec<&str> = p.published.iter().map(|p| p.name.as_str()).collect();
                    println!("  publishes: {}", names.join(", "));
                }
            }
            Ok(())
        }
        Some("affected") => {
            let projects = affected(platform, &args[1..])?;
            let all: BTreeSet<&str> = projects.iter().map(|p| p.dir.as_str()).collect();
            let fingerprints: BTreeMap<String, String> =
                affected::fingerprints(&load(platform)?, &head(platform)?)?;
            println!(
                "{}",
                serde_json::to_string(&affected::jobs(&projects, &all, &fingerprints))?
            );
            Ok(())
        }
        Some("check") => check(platform, &args[1..]),
        Some("config") if args.len() == 1 => config(platform),
        Some("bootstrap") => match &args[1..] {
            [flag, dir] if flag == "--check" => {
                bootstrap::check(platform, Path::new(dir), load(platform))
            }
            [dir] => bootstrap::build(platform, Path::new(dir), &load(platform)?),
            _ => Err(USAGE.into()),
        },
        Some("release") => release(platform, &args[1..]),
        Some("generate") => {
            let dump = match &args[1..] {
                [] => generate::DUMP_DIR,
                [flag, dir] if flag == "--dump" => dir.as_str(),
                _ => return Err(USAGE.into()),
            };
            generate::generate(platform, &root(platform)?, Path::new(dump))
        }
        Some("roundtrip") => roundtrip::roundtrip(platform, &root(platform)?, &args[1..]),
        Some(_) => Err(USAGE.into()),
    }
}

fn main() -> std::process::ExitCode {
    let platform = agplatform::platform();
    let args: Vec<String> = platform
        .env()
        .args()
        .skip(1)
        .map(|arg| arg.to_string_lossy().into_owned())
        .collect();

    match run(&platform, &args) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error: {}", e.description);
            std::process::ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(args: &[&str]) -> Vec<String> {
        args.iter().map(|a| a.to_string()).collect()
    }

    #[test]
    fn check_args_clean_force_and_projects() {
        let run = |force: bool, dirs: &[&str]| CheckArgs::Run {
            force,
            dirs: args(dirs),
        };

        assert_eq!(check_args(&[]).unwrap(), run(false, &[]));
        assert_eq!(
            check_args(&args(&["-p", "agdb", "--force", "-p", "agdb_api/php"])).unwrap(),
            run(true, &["agdb", "agdb_api/php"])
        );
        assert_eq!(check_args(&args(&["--clean"])).unwrap(), CheckArgs::Clean);
        assert!(check_args(&args(&["--clean", "--force"])).is_err());
        assert!(check_args(&args(&["agdb"])).is_err());
        assert!(check_args(&args(&["-p"])).is_err());
    }
}
