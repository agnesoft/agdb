mod affected;
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

use crate::ci_error::CIError;
use crate::project::Project;
use std::path::Path;
use std::process::Command;

const USAGE: &str = "usage:
  agci                         update versions and regenerate the API clients
  agci projects                list the projects and their dependencies
  agci affected [--base <ref>] JSON job matrix of the projects affected by the changes since <ref>
                               (merge base), or by the uncommitted changes without --base
  agci affected --all          JSON job matrix of all projects
  agci check [<project>...]    run the checks of the projects, default: the affected ones; all of
                               them run, failing ones are listed at the end
  agci release (--base <ref>|--all|--projects [<project>...]) [--plan]
                               after a push to main: release_dev of the projects affected since <ref>
                               (or of the given ones) and, if there is no v<version> tag for the root
                               Cargo.toml version, the release (release of all projects, then the
                               GitHub release and tag); --plan only prints the steps";

fn root() -> Result<String, CIError> {
    Ok(
        utilities::run_quiet(Command::new("git").args(["rev-parse", "--show-toplevel"]))?
            .trim()
            .to_string(),
    )
}

fn load() -> Result<Vec<Project>, CIError> {
    project::projects(manifest::manifests()?, |p| Path::new(p).exists())
}

fn changed(base: Option<&str>) -> Result<Vec<String>, CIError> {
    let mut files = match base {
        Some(base) => git::changed_since(base)?,
        None => git::uncommitted()?,
    };
    files.sort();
    files.dedup();
    Ok(files)
}

/// Whether the version of the crate is already on crates.io.
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
    let version = git::new_version()?;
    let release = release::Release {
        affected: affected_dirs.iter().map(String::as_str).collect(),
        version: (!git::tag_exists(&format!("v{version}"))?).then_some(version),
        sha: git::head()?,
    };
    let steps = release::plan(&projects, &release, published_crate)?;

    if plan_only {
        for step in &steps {
            println!("{step}");
        }
        Ok(())
    } else {
        check::run_steps("release", &steps, &root()?)
    }
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
        [base, r] if base == "--base" => affected::affected(&projects, &changed(Some(r))?)?
            .into_iter()
            .map(str::to_string)
            .collect(),
        [] => affected::affected(&projects, &changed(None)?)?
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
                    "{} ({:?}: {}) -> [{}]",
                    p.dir,
                    p.lang,
                    p.packages.join(", "),
                    p.depends.iter().cloned().collect::<Vec<_>>().join(", ")
                );
            }
            Ok(())
        }
        Some("affected") => {
            let projects = affected(&args[1..])?;
            let all: std::collections::BTreeSet<&str> =
                projects.iter().map(|p| p.dir.as_str()).collect();
            println!(
                "{}",
                serde_json::to_string(&affected::jobs(&projects, &all))?
            );
            Ok(())
        }
        Some("check") => {
            let root = root()?;
            let projects = load()?;
            let selected: Vec<&Project> = if args.len() > 1 {
                select(&projects, &args[1..])?
            } else {
                let changed = changed(None)?;
                let affected = affected::affected(&projects, &changed)?;
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
