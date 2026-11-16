use crate::CIError;
use crate::manifest::Dep;
use crate::manifest::Manifest;
use crate::manifest::manifest_file;
use crate::manifest::moved;
use crate::manifest::nested;
use crate::project_type::Details;
use crate::project_type::cargo::CrateDetails;
use crate::project_type::cargo::Targets;
use crate::utilities;
use agplatform::Platform;
use serde::Deserialize;
use std::path::PathBuf;
use std::process::Command;

/// The workspace members, every tracked `Cargo.toml` (but the root one) must be one.
pub(super) fn manifests(
    platform: &impl Platform,
    tracked: &[&str],
) -> Result<Vec<Manifest>, CIError> {
    let manifests = from_metadata(&metadata(platform)?)?;

    for (_, file) in nested(tracked, "Cargo.toml") {
        if !manifests
            .iter()
            .any(|m| manifest_file(&m.dir, "Cargo.toml") == file)
        {
            return Err(format!("'{file}' is not a member of the cargo workspace").into());
        }
    }

    Ok(manifests)
}

pub(super) fn target_dir(platform: &impl Platform) -> Result<PathBuf, CIError> {
    let metadata: serde_json::Value = serde_json::from_str(&metadata(platform)?)?;
    Ok(PathBuf::from(
        metadata["target_directory"]
            .as_str()
            .ok_or("cargo metadata without target_directory")?,
    ))
}

fn metadata(platform: &impl Platform) -> Result<String, CIError> {
    utilities::run_quiet(
        platform,
        Command::new("cargo").args(["metadata", "--no-deps", "--format-version", "1"]),
    )
}

#[derive(Deserialize)]
struct CargoMetadata {
    workspace_root: String,
    packages: Vec<CargoPackage>,
}

#[derive(Deserialize)]
struct CargoPackage {
    name: String,
    manifest_path: String,
    dependencies: Vec<CargoDependency>,
    metadata: Option<CargoPackageMetadata>,
    /// `null` for any registry, empty for `publish = false`.
    publish: Option<Vec<String>>,
    #[serde(default)]
    targets: Vec<CargoTarget>,
}

#[derive(Deserialize)]
struct CargoTarget {
    kind: Vec<String>,
}

#[derive(Deserialize)]
struct CargoPackageMetadata {
    agci: Option<serde_json::Value>,
}

#[derive(Deserialize)]
struct CargoDependency {
    path: Option<String>,
}

fn from_metadata(metadata: &str) -> Result<Vec<Manifest>, CIError> {
    let metadata: CargoMetadata = serde_json::from_str(metadata)?;
    let root = metadata.workspace_root;

    metadata
        .packages
        .into_iter()
        .map(|p| {
            let manifest = relative(&root, &p.manifest_path);

            if p.metadata.is_some_and(|m| m.agci.is_some()) {
                return Err(moved(&manifest));
            }

            let dir = manifest
                .trim_end_matches("Cargo.toml")
                .trim_end_matches('/')
                .to_string();
            Ok(Manifest {
                deps: p
                    .dependencies
                    .into_iter()
                    .filter_map(|d| d.path.map(|path| Dep::Dir(relative(&root, &path))))
                    .filter(|d| *d != Dep::Dir(dir.clone()))
                    .collect(),
                dir,
                name: p.name,
                publish: p.publish.is_none_or(|registries| !registries.is_empty()),
                details: Details::Cargo(CrateDetails {
                    targets: {
                        let has = |kinds: &[&str]| {
                            p.targets
                                .iter()
                                .flat_map(|t| &t.kind)
                                .any(|k| kinds.contains(&k.as_str()))
                        };
                        Targets {
                            library: has(&["lib", "rlib", "dylib", "proc-macro"]),
                            examples: has(&["example"]),
                            benches: has(&["bench"]),
                        }
                    },
                }),
            })
        })
        .collect()
}

fn relative(root: &str, path: &str) -> String {
    let root = root.replace('\\', "/");
    let path = path.replace('\\', "/");
    path.strip_prefix(&root)
        .unwrap_or(&path)
        .trim_start_matches('/')
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn targets(manifest: &Manifest) -> Targets {
        match manifest.details {
            Details::Cargo(CrateDetails { targets }) => targets,
            _ => unreachable!(),
        }
    }

    #[test]
    fn manifests_from_cargo_metadata() {
        let metadata = r#"{
            "workspace_root": "/repo",
            "packages": [
                {
                    "name": "server",
                    "manifest_path": "/repo/server/Cargo.toml",
                    "dependencies": [
                        { "name": "db", "path": "/repo/db" },
                        { "name": "server", "path": "/repo/server" },
                        { "name": "serde" }
                    ],
                    "metadata": { "docs": {} },
                    "publish": null,
                    "targets": [{ "kind": ["bin"] }, { "kind": ["custom-build"] }]
                },
                {
                    "name": "db",
                    "manifest_path": "/repo/db/Cargo.toml",
                    "dependencies": [],
                    "metadata": null,
                    "publish": [],
                    "targets": [{ "kind": ["lib"] }, { "kind": ["example"] }, { "kind": ["bench"] }]
                }
            ]
        }"#;

        let manifests = from_metadata(metadata).unwrap();

        assert_eq!(manifests.len(), 2);
        assert_eq!(manifests[0].dir, "server");
        assert_eq!(manifests[0].deps, vec![Dep::Dir("db".to_string())]);
        assert!(manifests[0].publish);
        assert!(!targets(&manifests[0]).library);
        assert!(!targets(&manifests[0]).examples);
        assert!(!targets(&manifests[0]).benches);
        assert_eq!(manifests[1].name, "db");
        assert!(!manifests[1].publish);
        assert!(targets(&manifests[1]).library);
        assert!(targets(&manifests[1]).examples);
        assert!(targets(&manifests[1]).benches);
    }

    #[test]
    fn manifest_with_agci_metadata_fails() {
        let metadata = r#"{
            "workspace_root": "/repo",
            "packages": [{
                "name": "a",
                "manifest_path": "/repo/a/Cargo.toml",
                "dependencies": [],
                "metadata": { "agci": { "checks": ["test"] } }
            }]
        }"#;

        assert_eq!(
            from_metadata(metadata).unwrap_err().description,
            "'a/Cargo.toml' has agci metadata, it belongs to 'agci.toml' now"
        );
    }

    #[test]
    fn manifests_windows_paths() {
        let metadata = r#"{
            "workspace_root": "C:\\repo",
            "packages": [{
                "name": "a",
                "manifest_path": "C:\\repo\\libs\\a\\Cargo.toml",
                "dependencies": [{ "name": "b", "path": "C:\\repo\\b" }],
                "metadata": null
            }]
        }"#;

        let manifests = from_metadata(metadata).unwrap();

        assert_eq!(manifests[0].dir, "libs/a");
        assert_eq!(manifests[0].deps, vec![Dep::Dir("b".to_string())]);
    }
}
