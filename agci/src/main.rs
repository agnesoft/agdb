mod affected;
mod check;
mod ci_error;
mod config;
mod git;
mod language;
mod manifest;
mod project;
mod publish;
mod release;
mod utilities;

use crate::ci_error::CIError;
use crate::project::Project;
use std::path::Path;
use std::process::Command;

const USAGE: &str = "usage:
  agci                         update versions and regenerate the API clients (release)
  agci projects                list the projects and their dependencies
  agci affected [--base <ref>] JSON job matrix of the projects affected by the changes since <ref>
                               (merge base), or by the uncommitted changes without --base
  agci affected --all          JSON job matrix of all projects
  agci check [<project>...]    run the checks of the projects, default: the affected ones
  agci publish (--base <ref>|--all) [--plan]
                               after a push to main: publish_dev of the projects affected since <ref>
                               and, if there is no v<version> tag for the root Cargo.toml version, the
                               release (publish of all projects, then the GitHub release and tag);
                               --plan only prints the steps";

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

fn publish(args: &[String]) -> Result<(), CIError> {
    let (plan_only, args) = match args.split_last() {
        Some((last, rest)) if last == "--plan" => (true, rest),
        _ => (false, args),
    };

    if args.is_empty() {
        return Err(USAGE.into());
    }

    let projects = load()?;
    let affected_dirs: Vec<String> = affected(args)?.into_iter().map(|p| p.dir).collect();
    let version = git::new_version()?;
    let publish = publish::Publish {
        affected: affected_dirs.iter().map(String::as_str).collect(),
        release: (!git::tag_exists(&format!("v{version}"))?).then_some(version),
        sha: git::head()?,
    };
    let steps = publish::plan(&projects, &publish, published_crate)?;

    if plan_only {
        for step in &steps {
            println!("{step}");
        }
        Ok(())
    } else {
        check::run_steps("publish", &steps, &root()?)
    }
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
        None => release::release(),
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
                args[1..]
                    .iter()
                    .map(|dir| {
                        let dir = dir.trim_end_matches('/');
                        projects
                            .iter()
                            .find(|p| p.dir == dir)
                            .ok_or(CIError::from(format!("unknown project '{dir}'")))
                    })
                    .collect::<Result<_, _>>()?
            } else {
                let changed = changed(None)?;
                let affected = affected::affected(&projects, &changed)?;
                projects
                    .iter()
                    .filter(|p| affected.contains(p.dir.as_str()))
                    .collect()
            };

            for project in selected {
                check::check(project, &projects, &root)?;
            }

            Ok(())
        }
        Some("publish") => publish(&args[1..]),
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
