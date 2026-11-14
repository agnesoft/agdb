use crate::CIError;
use crate::utilities::run_command;
use std::path::Path;
use std::process::Command;

/// The latest `v*` tag without the `v`. Shallow clones (CI) have no tags, then the remote ones are used.
pub(crate) fn current_version() -> Result<String, CIError> {
    let local =
        run_command(Command::new("git").args(["tag", "--list", "--sort=taggerdate", "v*"]))?;
    let tag = match local.trim().lines().last() {
        Some(tag) => tag.to_string(),
        None => remote_tags("v*")?.pop().ok_or("tags not found")?,
    };
    Ok(tag[1..].to_string())
}

/// Tags of `origin` matching the pattern, in version order.
fn remote_tags(pattern: &str) -> Result<Vec<String>, CIError> {
    Ok(crate::utilities::run_quiet(Command::new("git").args([
        "ls-remote",
        "--tags",
        "--refs",
        "--sort=v:refname",
        "origin",
        &format!("refs/tags/{pattern}"),
    ]))?
    .lines()
    .filter_map(|line| {
        line.split_once("refs/tags/")
            .map(|(_, tag)| tag.to_string())
    })
    .collect())
}

pub(crate) fn new_version() -> Result<String, CIError> {
    Ok(std::fs::read_to_string(Path::new("Cargo.toml"))?
        .lines()
        .find(|line| line.trim_start().starts_with("version"))
        .and_then(|line| line.trim_start().split('"').nth(1).map(|v| v.to_string()))
        .ok_or("version not found")?)
}

/// Files changed on HEAD since its merge base with `base`, both sides of renames.
pub(crate) fn changed_since(base: &str) -> Result<Vec<String>, CIError> {
    Ok(crate::utilities::run_quiet(Command::new("git").args([
        "diff",
        "--name-only",
        "--no-renames",
        &format!("{base}...HEAD"),
    ]))?
    .lines()
    .map(str::to_string)
    .collect())
}

/// Uncommitted (staged, unstaged and untracked) files.
pub(crate) fn uncommitted() -> Result<Vec<String>, CIError> {
    let mut files: Vec<String> = crate::utilities::run_quiet(Command::new("git").args([
        "diff",
        "--name-only",
        "--no-renames",
        "HEAD",
    ]))?
    .lines()
    .map(str::to_string)
    .collect();
    files.extend(
        crate::utilities::run_quiet(Command::new("git").args([
            "ls-files",
            "--others",
            "--exclude-standard",
        ]))?
        .lines()
        .map(str::to_string),
    );
    Ok(files)
}

/// Locally or on `origin` (shallow clones have no tags).
pub(crate) fn tag_exists(tag: &str) -> Result<bool, CIError> {
    let local = !crate::utilities::run_quiet(Command::new("git").args(["tag", "--list", tag]))?
        .trim()
        .is_empty();
    Ok(local || remote_tags(tag)?.iter().any(|t| t == tag))
}

pub(crate) fn head() -> Result<String, CIError> {
    Ok(
        crate::utilities::run_quiet(Command::new("git").args(["rev-parse", "HEAD"]))?
            .trim()
            .to_string(),
    )
}
