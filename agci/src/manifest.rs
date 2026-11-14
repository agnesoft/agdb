use crate::CIError;
use crate::config::Config;
use crate::utilities;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::path::Path;
use std::process::Command;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Lang {
    Node,
    Rust,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Dep {
    Dir(String),
    Name(String),
}

#[derive(Debug)]
pub(crate) struct Manifest {
    pub(crate) dir: String,
    pub(crate) lang: Lang,
    pub(crate) name: String,
    pub(crate) deps: Vec<Dep>,
    pub(crate) scripts: BTreeSet<String>,
    pub(crate) config: Option<Config>,
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
}

#[derive(Deserialize)]
struct CargoPackageMetadata {
    agci: Option<Config>,
}

#[derive(Deserialize)]
struct CargoDependency {
    path: Option<String>,
}

#[derive(Deserialize)]
struct PackageJson {
    name: Option<String>,
    #[serde(default)]
    dependencies: BTreeMap<String, String>,
    #[serde(default, rename = "devDependencies")]
    dev_dependencies: BTreeMap<String, String>,
    #[serde(default)]
    scripts: BTreeMap<String, String>,
    agci: Option<Config>,
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
    let mut manifests = rust_manifests(&metadata)?;
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
            manifests.push(node_manifest(dir, &std::fs::read_to_string(file)?)?);
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

fn rust_manifests(metadata: &str) -> Result<Vec<Manifest>, CIError> {
    let metadata: CargoMetadata = serde_json::from_str(metadata)?;
    let root = metadata.workspace_root;

    Ok(metadata
        .packages
        .into_iter()
        .map(|p| {
            let manifest = relative(&root, &p.manifest_path);
            let dir = manifest
                .trim_end_matches("Cargo.toml")
                .trim_end_matches('/')
                .to_string();
            Manifest {
                deps: p
                    .dependencies
                    .into_iter()
                    .filter_map(|d| d.path.map(|path| Dep::Dir(relative(&root, &path))))
                    .filter(|d| *d != Dep::Dir(dir.clone()))
                    .collect(),
                dir,
                lang: Lang::Rust,
                name: p.name,
                scripts: BTreeSet::new(),
                config: p.metadata.and_then(|m| m.agci),
            }
        })
        .collect())
}

pub(crate) fn node_manifest(dir: &str, content: &str) -> Result<Manifest, CIError> {
    let package: PackageJson = serde_json::from_str(content)
        .map_err(|e| format!("'{}': {e}", manifest_file(dir, "package.json")))?;
    let name = package.name.ok_or(format!(
        "'{}' has no name",
        manifest_file(dir, "package.json")
    ))?;
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
        lang: Lang::Node,
        name,
        deps,
        scripts: package.scripts.into_keys().collect(),
        config: package.agci,
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
    fn rust_manifests_from_cargo_metadata() {
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
                    "metadata": { "agci": { "checks": ["format"], "depends": ["ui"] } }
                },
                {
                    "name": "db",
                    "manifest_path": "/repo/db/Cargo.toml",
                    "dependencies": [],
                    "metadata": null
                }
            ]
        }"#;

        let manifests = rust_manifests(metadata).unwrap();

        assert_eq!(manifests.len(), 2);
        assert_eq!(manifests[0].dir, "server");
        assert_eq!(manifests[0].deps, vec![Dep::Dir("db".to_string())]);
        let config = manifests[0].config.as_ref().unwrap();
        assert_eq!(config.checks, vec!["format"]);
        assert_eq!(config.depends, vec!["ui"]);
        assert!(manifests[1].config.is_none());
    }

    #[test]
    fn rust_manifests_windows_paths() {
        let metadata = r#"{
            "workspace_root": "C:\\repo",
            "packages": [{
                "name": "a",
                "manifest_path": "C:\\repo\\libs\\a\\Cargo.toml",
                "dependencies": [{ "name": "b", "path": "C:\\repo\\b" }],
                "metadata": null
            }]
        }"#;

        let manifests = rust_manifests(metadata).unwrap();

        assert_eq!(manifests[0].dir, "libs/a");
        assert_eq!(manifests[0].deps, vec![Dep::Dir("b".to_string())]);
    }

    #[test]
    fn node_manifest_local_dependencies() {
        let content = r#"{
            "name": "client",
            "scripts": { "build": "tsc", "test": "vitest" },
            "dependencies": { "api": "workspace:*", "axios": "^1", "lib": "file:../../lib" },
            "devDependencies": { "tsconfig": "workspace:^", "eslint": "catalog:" },
            "agci": { "checks": ["build"] }
        }"#;

        let manifest = node_manifest("examples/client", content).unwrap();

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
        assert_eq!(manifest.config.unwrap().checks, vec!["build"]);
    }

    #[test]
    fn node_manifest_requires_name() {
        let error = node_manifest("php", r#"{ "scripts": {} }"#).unwrap_err();

        assert_eq!(error.description, "'php/package.json' has no name");
    }

    #[test]
    fn node_manifest_rejects_unknown_agci_field() {
        let error = node_manifest("a", r#"{ "name": "a", "agci": { "check": [] } }"#).unwrap_err();

        assert!(
            error.description.contains("unknown field `check`"),
            "{}",
            error.description
        );
    }
}
