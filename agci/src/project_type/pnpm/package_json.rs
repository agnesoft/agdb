use crate::CIError;
use crate::manifest::Dep;
use crate::manifest::Manifest;
use crate::manifest::manifest_file;
use crate::manifest::moved;
use crate::project_type::Details;
use crate::project_type::pnpm::PackageDetails;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::Path;

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
pub(super) fn root_checks(content: &str) -> Result<Option<Vec<String>>, CIError> {
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

pub(super) fn from_package_json(dir: &str, content: &str) -> Result<Manifest, CIError> {
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
        name: name.clone(),
        deps,
        publish: !package.private,
        details: Details::Pnpm(PackageDetails {
            name,
            file,
            scripts: package.scripts.into_keys().collect(),
            version: package.version,
        }),
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
    use std::collections::BTreeSet;

    fn details(manifest: &Manifest) -> &PackageDetails {
        match &manifest.details {
            Details::Pnpm(details) => details,
            _ => unreachable!(),
        }
    }

    #[test]
    fn manifest_local_dependencies() {
        let content = r#"{
            "name": "client",
            "version": "1.2.3",
            "scripts": { "build": "tsc", "test": "vitest" },
            "dependencies": { "api": "workspace:*", "axios": "^1", "lib": "file:../../lib" },
            "devDependencies": { "tsconfig": "workspace:^", "eslint": "catalog:" }
        }"#;

        let manifest = from_package_json("examples/client", content).unwrap();

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
            details(&manifest).scripts,
            BTreeSet::from(["build".to_string(), "test".to_string()])
        );
        assert!(manifest.publish);
        assert_eq!(details(&manifest).version.as_deref(), Some("1.2.3"));
    }

    #[test]
    fn manifest_published_without_version_fails() {
        assert_eq!(
            from_package_json("a", r#"{ "name": "a" }"#)
                .unwrap_err()
                .description,
            "'a/package.json' is published (not private) but has no version"
        );
        assert!(
            !from_package_json("a", r#"{ "name": "a", "private": true }"#)
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
    fn manifest_requires_name() {
        let error = from_package_json("php", r#"{ "private": true }"#).unwrap_err();

        assert_eq!(error.description, "'php/package.json' has no name");
    }

    #[test]
    fn manifest_with_agci_metadata_fails() {
        assert_eq!(
            from_package_json(
                "a",
                r#"{ "name": "a", "private": true, "agci": { "checks": [] } }"#
            )
            .unwrap_err()
            .description,
            "'a/package.json' has agci metadata, it belongs to 'agci.toml' now"
        );
    }
}
