use crate::CIError;
use crate::config::FILE;
use crate::utilities;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::path::Path;
use std::process::Command;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
/// What drives the project: its manifest and the tool running it.
pub(crate) enum Kind {
    /// A Cargo workspace member.
    Cargo,
    /// A package.json package (pnpm workspace, turbo).
    Pnpm,
    /// A directory without a manifest, its checks are script files.
    Shell,
}

/// As in `[defaults.<kind>]`.
impl std::fmt::Display for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Kind::Cargo => "cargo",
            Kind::Pnpm => "pnpm",
            Kind::Shell => "shell",
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Dep {
    Dir(String),
    Name(String),
}

#[derive(Debug)]
pub(crate) struct Manifest {
    pub(crate) dir: String,
    pub(crate) kind: Kind,
    pub(crate) name: String,
    pub(crate) deps: Vec<Dep>,
    pub(crate) scripts: BTreeSet<String>,
    /// Released to its registry (crates.io, npm): Cargo `publish` is not `false`, package.json is not
    /// `private`.
    pub(crate) publish: bool,
    /// package.json only, Cargo crates have the workspace version.
    pub(crate) version: Option<String>,
    /// Cargo: has a library target (doc tests).
    pub(crate) library: bool,
    /// Cargo: has example targets.
    pub(crate) examples: bool,
    /// Cargo: has bench targets.
    pub(crate) benches: bool,
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

#[derive(Deserialize)]
struct PackageJson {
    name: Option<String>,
    version: Option<String>,
    #[serde(default)]
    private: bool,
    #[serde(default)]
    dependencies: BTreeMap<String, String>,
    #[serde(default, rename = "devDependencies")]
    dev_dependencies: BTreeMap<String, String>,
    #[serde(default)]
    scripts: BTreeMap<String, String>,
    agci: Option<serde_json::Value>,
}

/// The tasks of the root package.json `check` script (`turbo run <task>...`), the default checks of
/// the pnpm projects.
pub(crate) fn root_checks(content: &str) -> Result<Option<Vec<String>>, CIError> {
    let package: PackageJson =
        serde_json::from_str(content).map_err(|e| format!("'package.json': {e}"))?;
    let Some(script) = package.scripts.get(CHECK_SCRIPT) else {
        return Ok(None);
    };

    match script.split_whitespace().collect::<Vec<_>>().as_slice() {
        ["turbo", "run", tasks @ ..]
            if !tasks.is_empty()
                && tasks
                    .iter()
                    .all(|t| !t.starts_with('-') && !t.contains('#')) =>
        {
            Ok(Some(tasks.iter().map(|t| t.to_string()).collect()))
        }
        _ => Err(format!(
            "'package.json' script '{CHECK_SCRIPT}' must be `turbo run <task>...`, got `{script}`"
        )
        .into()),
    }
}

const CHECK_SCRIPT: &str = "check";

fn moved(manifest: &str) -> CIError {
    format!("'{manifest}' has agci metadata, it belongs to '{FILE}' now").into()
}

pub(crate) fn manifests() -> Result<Vec<Manifest>, CIError> {
    let tracked = utilities::run_quiet(Command::new("git").args([
        "ls-files",
        "--",
        "Cargo.toml",
        "**/Cargo.toml",
        "package.json",
        "**/package.json",
    ]))?;
    let metadata = utilities::run_quiet(Command::new("cargo").args([
        "metadata",
        "--no-deps",
        "--format-version",
        "1",
    ]))?;
    let mut manifests = cargo_manifests(&metadata)?;
    let workspace: BTreeSet<String> = manifests
        .iter()
        .map(|m| manifest_file(&m.dir, "Cargo.toml"))
        .collect();

    for file in tracked.lines().filter(|f| f.ends_with("Cargo.toml")) {
        if file != "Cargo.toml" && !workspace.contains(file) {
            return Err(format!("'{file}' is not a member of the cargo workspace").into());
        }
    }

    for file in tracked.lines().filter(|f| f.ends_with("package.json")) {
        if file != "package.json" {
            let dir = file.trim_end_matches("package.json").trim_end_matches('/');
            manifests.push(pnpm_manifest(dir, &std::fs::read_to_string(file)?)?);
        }
    }

    Ok(manifests)
}

fn manifest_file(dir: &str, file: &str) -> String {
    if dir.is_empty() {
        file.to_string()
    } else {
        format!("{dir}/{file}")
    }
}

pub(crate) fn relative(root: &str, path: &str) -> String {
    let root = root.replace('\\', "/");
    let path = path.replace('\\', "/");
    path.strip_prefix(&root)
        .unwrap_or(&path)
        .trim_start_matches('/')
        .to_string()
}

fn cargo_manifests(metadata: &str) -> Result<Vec<Manifest>, CIError> {
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
                kind: Kind::Cargo,
                name: p.name,
                scripts: BTreeSet::new(),
                publish: p.publish.is_none_or(|registries| !registries.is_empty()),
                version: None,
                library: p
                    .targets
                    .iter()
                    .flat_map(|t| &t.kind)
                    .any(|k| ["lib", "rlib", "dylib", "proc-macro"].contains(&k.as_str())),
                examples: p
                    .targets
                    .iter()
                    .flat_map(|t| &t.kind)
                    .any(|k| k == "example"),
                benches: p.targets.iter().flat_map(|t| &t.kind).any(|k| k == "bench"),
            })
        })
        .collect()
}

pub(crate) fn pnpm_manifest(dir: &str, content: &str) -> Result<Manifest, CIError> {
    let package: PackageJson = serde_json::from_str(content)
        .map_err(|e| format!("'{}': {e}", manifest_file(dir, "package.json")))?;
    let file = manifest_file(dir, "package.json");

    if package.agci.is_some() {
        return Err(moved(&file));
    }

    let name = package.name.ok_or(format!("'{file}' has no name"))?;

    if !package.private && package.version.is_none() {
        return Err(format!("'{file}' is published (not private) but has no version").into());
    }

    let mut deps = Vec::new();

    for (dep, version) in package.dependencies.iter().chain(&package.dev_dependencies) {
        if version.starts_with("workspace:") {
            deps.push(Dep::Name(dep.clone()));
        } else if let Some(path) = version
            .strip_prefix("file:")
            .or(version.strip_prefix("link:"))
        {
            deps.push(Dep::Dir(normalize(
                &Path::new(dir).join(path).to_string_lossy(),
            )));
        }
    }

    Ok(Manifest {
        dir: dir.to_string(),
        kind: Kind::Pnpm,
        name,
        deps,
        scripts: package.scripts.into_keys().collect(),
        publish: !package.private,
        version: package.version,
        library: false,
        examples: false,
        benches: false,
    })
}

fn normalize(path: &str) -> String {
    let mut parts: Vec<&str> = Vec::new();

    let path = path.replace('\\', "/");

    for part in path.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            p => parts.push(p),
        }
    }

    parts.join("/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cargo_manifests_from_cargo_metadata() {
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

        let manifests = cargo_manifests(metadata).unwrap();

        assert_eq!(manifests.len(), 2);
        assert_eq!(manifests[0].dir, "server");
        assert_eq!(manifests[0].deps, vec![Dep::Dir("db".to_string())]);
        assert!(manifests[0].publish);
        assert!(!manifests[0].library);
        assert!(!manifests[0].examples);
        assert!(!manifests[0].benches);
        assert_eq!(manifests[1].name, "db");
        assert!(!manifests[1].publish);
        assert!(manifests[1].library);
        assert!(manifests[1].examples);
        assert!(manifests[1].benches);
    }

    #[test]
    fn cargo_manifest_with_agci_metadata_fails() {
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
            cargo_manifests(metadata).unwrap_err().description,
            "'a/Cargo.toml' has agci metadata, it belongs to 'agci.toml' now"
        );
    }

    #[test]
    fn cargo_manifests_windows_paths() {
        let metadata = r#"{
            "workspace_root": "C:\\repo",
            "packages": [{
                "name": "a",
                "manifest_path": "C:\\repo\\libs\\a\\Cargo.toml",
                "dependencies": [{ "name": "b", "path": "C:\\repo\\b" }],
                "metadata": null
            }]
        }"#;

        let manifests = cargo_manifests(metadata).unwrap();

        assert_eq!(manifests[0].dir, "libs/a");
        assert_eq!(manifests[0].deps, vec![Dep::Dir("b".to_string())]);
    }

    #[test]
    fn pnpm_manifest_local_dependencies() {
        let content = r#"{
            "name": "client",
            "version": "1.2.3",
            "scripts": { "build": "tsc", "test": "vitest" },
            "dependencies": { "api": "workspace:*", "axios": "^1", "lib": "file:../../lib" },
            "devDependencies": { "tsconfig": "workspace:^", "eslint": "catalog:" }
        }"#;

        let manifest = pnpm_manifest("examples/client", content).unwrap();

        assert_eq!(manifest.name, "client");
        assert_eq!(
            manifest.deps,
            vec![
                Dep::Name("api".to_string()),
                Dep::Dir("lib".to_string()),
                Dep::Name("tsconfig".to_string())
            ]
        );
        assert_eq!(
            manifest.scripts,
            BTreeSet::from(["build".to_string(), "test".to_string()])
        );
        assert!(manifest.publish);
        assert_eq!(manifest.version.as_deref(), Some("1.2.3"));
    }

    #[test]
    fn pnpm_manifest_published_without_version_fails() {
        assert_eq!(
            pnpm_manifest("a", r#"{ "name": "a" }"#)
                .unwrap_err()
                .description,
            "'a/package.json' is published (not private) but has no version"
        );
        assert!(
            !pnpm_manifest("a", r#"{ "name": "a", "private": true }"#)
                .unwrap()
                .publish
        );
    }

    #[test]
    fn root_check_script_tasks() {
        let root =
            |script: &str| root_checks(&format!(r#"{{ "scripts": {{ "check": "{script}" }} }}"#));

        assert_eq!(
            root("turbo run openapi format:check  test").unwrap(),
            Some(vec![
                "openapi".to_string(),
                "format:check".to_string(),
                "test".to_string()
            ])
        );
        assert_eq!(root_checks(r#"{ "scripts": {} }"#).unwrap(), None);

        for script in [
            "turbo run",
            "turbo run test --force",
            "turbo run a#test",
            "pnpm test",
        ] {
            assert!(
                root(script)
                    .unwrap_err()
                    .description
                    .contains("must be `turbo run <task>...`"),
                "{script}"
            );
        }
    }

    #[test]
    fn pnpm_manifest_requires_name() {
        let error = pnpm_manifest("php", r#"{ "private": true }"#).unwrap_err();

        assert_eq!(error.description, "'php/package.json' has no name");
    }

    #[test]
    fn pnpm_manifest_with_agci_metadata_fails() {
        assert_eq!(
            pnpm_manifest(
                "a",
                r#"{ "name": "a", "private": true, "agci": { "checks": [] } }"#
            )
            .unwrap_err()
            .description,
            "'a/package.json' has agci metadata, it belongs to 'agci.toml' now"
        );
    }
}
