use crate::CIError;
use crate::utilities::run_command;
use crate::utilities::run_quiet;
use crate::utilities::run_quiet_with_input;
use agplatform::Fs;
use agplatform::Platform;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::Command;

/// The latest `v*` tag without the `v`. Shallow clones (CI) have no tags, then the remote ones are used.
pub(crate) fn current_version(platform: &impl Platform) -> Result<String, CIError> {
    let local = run_command(
        platform,
        Command::new("git").args(["tag", "--list", "--sort=taggerdate", "v*"]),
    )?;
    let tag = match local.trim().lines().last() {
        Some(tag) => tag.to_string(),
        None => remote_tags(platform, "v*")?.pop().ok_or("tags not found")?,
    };
    Ok(tag[1..].to_string())
}

/// Tags of `origin` matching the pattern, in version order.
fn remote_tags(platform: &impl Platform, pattern: &str) -> Result<Vec<String>, CIError> {
    Ok(run_quiet(
        platform,
        Command::new("git").args([
            "ls-remote",
            "--tags",
            "--refs",
            "--sort=v:refname",
            "origin",
            &format!("refs/tags/{pattern}"),
        ]),
    )?
    .lines()
    .filter_map(|line| {
        line.split_once("refs/tags/")
            .map(|(_, tag)| tag.to_string())
    })
    .collect())
}

pub(crate) fn merge_base(platform: &impl Platform, base: &str) -> Result<String, CIError> {
    Ok(run_quiet(
        platform,
        Command::new("git").args(["merge-base", base, "HEAD"]),
    )?
    .trim()
    .to_string())
}

/// The files of the commit: path -> git object id.
pub(crate) fn files_at(
    platform: &impl Platform,
    rev: &str,
) -> Result<BTreeMap<String, String>, CIError> {
    let listing = run_quiet(
        platform,
        Command::new("git").args(["ls-tree", "-r", "-z", "--full-tree", rev]),
    )?;
    parse_tree(&listing)
}

/// `<mode> <type> <id>\t<path>` entries separated by NUL.
fn parse_tree(listing: &str) -> Result<BTreeMap<String, String>, CIError> {
    listing
        .split('\0')
        .filter(|entry| !entry.is_empty())
        .map(|entry| {
            let (meta, path) = entry
                .split_once('\t')
                .ok_or(format!("unexpected git ls-tree entry '{entry}'"))?;
            let id = meta.rsplit(' ').next().unwrap_or_default();
            Ok((path.to_string(), id.to_string()))
        })
        .collect()
}

pub(crate) fn file_at(platform: &impl Platform, rev: &str, path: &str) -> Result<String, CIError> {
    run_quiet(
        platform,
        Command::new("git").args(["show", &format!("{rev}:{path}")]),
    )
}

/// The files of HEAD with the uncommitted changes: a changed or new file gets the git object id of
/// its content (the one it gets when committed), a deleted one is left out.
pub(crate) fn worktree_files(
    platform: &impl Platform,
) -> Result<BTreeMap<String, String>, CIError> {
    let mut files = files_at(platform, "HEAD")?;
    let (present, deleted): (Vec<String>, Vec<String>) =
        uncommitted(platform)?.into_iter().partition(|path| {
            platform
                .fs()
                .metadata(path)
                .is_ok_and(|metadata| metadata.file_type.is_file())
        });

    for path in deleted {
        files.remove(&path);
    }

    if !present.is_empty() {
        let ids = run_quiet_with_input(
            platform,
            Command::new("git").args(["hash-object", "--stdin-paths"]),
            present.join("\n") + "\n",
        )?;
        files.extend(present.into_iter().zip(ids.lines().map(str::to_string)));
    }

    Ok(files)
}

/// Uncommitted (staged, unstaged and untracked) files.
pub(crate) fn uncommitted(platform: &impl Platform) -> Result<Vec<String>, CIError> {
    let mut files = Vec::new();

    for args in [
        &["diff", "--name-only", "--no-renames", "-z", "HEAD"][..],
        &["ls-files", "--others", "--exclude-standard", "-z"],
    ] {
        files.extend(
            run_quiet(platform, Command::new("git").args(args))?
                .split('\0')
                .filter(|path| !path.is_empty())
                .map(str::to_string),
        );
    }

    Ok(files)
}

/// Updates `<remote>/<branch>`.
pub(crate) fn fetch(platform: &impl Platform, remote: &str, branch: &str) -> Result<(), CIError> {
    run_quiet(
        platform,
        Command::new("git")
            .args(["fetch", "--quiet", remote, branch])
            .env("GIT_TERMINAL_PROMPT", "0"),
    )
    .map(|_| ())
}

/// The `.git` directory shared by all worktrees of the repository.
pub(crate) fn common_dir(platform: &impl Platform) -> Result<PathBuf, CIError> {
    Ok(PathBuf::from(
        run_quiet(
            platform,
            Command::new("git").args(["rev-parse", "--path-format=absolute", "--git-common-dir"]),
        )?
        .trim(),
    ))
}

/// Locally or on `origin` (shallow clones have no tags).
pub(crate) fn tag_exists(platform: &impl Platform, tag: &str) -> Result<bool, CIError> {
    let local = !run_quiet(platform, Command::new("git").args(["tag", "--list", tag]))?
        .trim()
        .is_empty();
    Ok(local || remote_tags(platform, tag)?.iter().any(|t| t == tag))
}

pub(crate) fn head(platform: &impl Platform) -> Result<String, CIError> {
    Ok(
        run_quiet(platform, Command::new("git").args(["rev-parse", "HEAD"]))?
            .trim()
            .to_string(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tree_entries_by_path() {
        let files = parse_tree(
            "100644 blob 1a2b\tCargo.toml\x00100755 blob 3c4d\tagdb server/run.sh\x00160000 commit 5e6f\tsub\x00",
        )
        .unwrap();

        assert_eq!(
            files,
            BTreeMap::from(
                [
                    ("Cargo.toml", "1a2b"),
                    ("agdb server/run.sh", "3c4d"),
                    ("sub", "5e6f")
                ]
                .map(|(p, i)| (p.to_string(), i.to_string()))
            )
        );
    }
}
