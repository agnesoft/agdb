use crate::CIError;
use crate::config::FILE;
use crate::project_type::Details;
use crate::project_type::Kind;
use crate::utilities;
use agplatform::Platform;
use std::process::Command;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Dep {
    /// The directory of a manifest.
    Dir(String),
    /// A package of the same project type.
    Name(String),
}

/// A package of the repository as its project type found it.
#[derive(Debug)]
pub(crate) struct Manifest {
    pub(crate) dir: String,
    pub(crate) name: String,
    /// The packages of the repository it depends on.
    pub(crate) deps: Vec<Dep>,
    /// Released to the registry of its type.
    pub(crate) publish: bool,
    pub(crate) details: Details,
}

impl Manifest {
    pub(crate) fn kind(&self) -> Kind {
        self.details.kind()
    }
}

/// The manifests of all project types among the tracked files.
pub(crate) fn manifests(platform: &impl Platform) -> Result<Vec<Manifest>, CIError> {
    let files = utilities::run_quiet(platform, Command::new("git").arg("ls-files"))?;
    let tracked: Vec<&str> = files.lines().collect();
    let mut manifests = Vec::new();

    for kind in Kind::ALL {
        manifests.extend(kind.manifests(platform, &tracked)?);
    }

    Ok(manifests)
}

/// The tracked files named `name` except the root one, with their directory.
pub(crate) fn nested<'a>(tracked: &[&'a str], name: &str) -> Vec<(&'a str, &'a str)> {
    tracked
        .iter()
        .filter_map(|file| {
            file.strip_suffix(name)
                .and_then(|dir| dir.strip_suffix('/'))
                .map(|dir| (dir, *file))
        })
        .collect()
}

pub(crate) fn moved(manifest: &str) -> CIError {
    format!("'{manifest}' has agci metadata, it belongs to '{FILE}' now").into()
}

pub(crate) fn manifest_file(dir: &str, file: &str) -> String {
    if dir.is_empty() {
        file.to_string()
    } else {
        format!("{dir}/{file}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nested_manifests_with_their_directory() {
        assert_eq!(
            nested(
                &[
                    "Cargo.toml",
                    "a/Cargo.toml",
                    "a/b/Cargo.toml",
                    "c/NotCargo.toml"
                ],
                "Cargo.toml"
            ),
            vec![("a", "a/Cargo.toml"), ("a/b", "a/b/Cargo.toml")]
        );
    }
}
