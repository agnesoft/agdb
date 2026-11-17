use crate::Error;
use agplatform::CommandExt;
use agplatform::Env;
use agplatform::Platform;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

/// A package of a cargo workspace and the crates cargo builds from it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Package {
    pub name: String,
    /// Its `Cargo.toml`.
    pub manifest: PathBuf,
    /// Sorted.
    pub targets: Vec<Target>,
    /// The features of its `[features]`, sorted.
    pub features: Vec<String>,
}

/// A crate of a package, what [`SourceCrate::load`](crate::SourceCrate::load)
/// takes.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Target {
    pub kind: TargetKind,
    pub root: PathBuf,
    /// The crates of its extern prelude as cargo passes them with `--extern`,
    /// apart from `std`, `core` and `alloc`.
    pub externs: Vec<String>,
}

/// In the order of [`Package::targets`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TargetKind {
    Lib,
    Bin,
    Test,
    Example,
    Bench,
    /// The build script.
    Build,
}

#[derive(Deserialize)]
struct Metadata {
    packages: Vec<MetadataPackage>,
}

#[derive(Deserialize)]
struct MetadataPackage {
    name: String,
    manifest_path: PathBuf,
    targets: Vec<MetadataTarget>,
    dependencies: Vec<MetadataDependency>,
    #[serde(default)]
    features: BTreeMap<String, Vec<String>>,
}

#[derive(Deserialize)]
struct MetadataTarget {
    name: String,
    kind: Vec<String>,
    src_path: PathBuf,
}

#[derive(Deserialize)]
struct MetadataDependency {
    name: String,
    rename: Option<String>,
    kind: Option<String>,
}

impl Target {
    /// A crate without dependencies, e.g. a single file.
    pub fn new(kind: TargetKind, root: impl Into<PathBuf>) -> Self {
        Self {
            kind,
            root: root.into(),
            externs: Vec::new(),
        }
    }
}

/// The packages of the workspace of `manifest` (a `Cargo.toml`) and their
/// crates, from `cargo metadata` (`$CARGO` when set, as for a cargo
/// subcommand). Each crate can name:
/// - the dependencies, dev and target-specific ones included (the build
///   script only the build dependencies);
/// - the package's own library, unless it is that library;
/// - `proc_macro` for a proc-macro library.
pub fn packages(p: &impl Platform, manifest: &Path) -> Result<Vec<Package>, Error> {
    let cargo = p.env().var("CARGO").unwrap_or_else(|| "cargo".into());
    let error = |message: String| Error::Cargo {
        path: manifest.to_path_buf(),
        message,
    };
    let output = Command::new(cargo)
        .args([
            "metadata",
            "--no-deps",
            "--format-version",
            "1",
            "--manifest-path",
        ])
        .arg(manifest)
        .run_blocking(p.process())
        .map_err(|e| {
            error(match e.command_output() {
                Some(output) => output.stderr_lossy().trim().to_string(),
                None => e.to_string(),
            })
        })?;
    let metadata: Metadata =
        serde_json::from_slice(&output.stdout).map_err(|e| error(e.to_string()))?;

    Ok(metadata.packages.into_iter().map(package).collect())
}

fn package(package: MetadataPackage) -> Package {
    let lib = package
        .targets
        .iter()
        .find(|target| kind(&target.kind) == TargetKind::Lib)
        .map(|lib| crate_name(&lib.name));
    let dependencies = |build: bool| {
        package
            .dependencies
            .iter()
            .filter(|d| (d.kind.as_deref() == Some("build")) == build)
            .map(|d| crate_name(d.rename.as_deref().unwrap_or(&d.name)))
            .collect::<Vec<_>>()
    };
    let mut targets: Vec<Target> = package
        .targets
        .iter()
        .map(|target| {
            let kind = kind(&target.kind);
            let mut externs = dependencies(kind == TargetKind::Build);

            if kind == TargetKind::Lib {
                if target.kind.iter().any(|k| k == "proc-macro") {
                    externs.push("proc_macro".to_string());
                }
            } else if let Some(lib) = &lib {
                externs.push(lib.clone());
            }

            Target {
                kind,
                root: target.src_path.clone(),
                externs,
            }
        })
        .collect();
    targets.sort();

    Package {
        name: package.name,
        manifest: package.manifest_path,
        targets,
        features: package.features.into_keys().collect(),
    }
}

/// Cargo's target kinds, any crate type of a library being a library.
fn kind(kinds: &[String]) -> TargetKind {
    match kinds.first().map(String::as_str) {
        Some("bin") => TargetKind::Bin,
        Some("test") => TargetKind::Test,
        Some("example") => TargetKind::Example,
        Some("bench") => TargetKind::Bench,
        Some("custom-build") => TargetKind::Build,
        _ => TargetKind::Lib,
    }
}

fn crate_name(name: &str) -> String {
    name.replace('-', "_")
}

#[cfg(test)]
mod tests {
    use super::*;
    use agplatform::CommandOutput;
    use agplatform::TestEnv;
    use agplatform::test_platform::TestPlatform;
    use agplatform::test_platform::test_platform;

    fn platform(metadata: &str) -> TestPlatform {
        let platform = test_platform();
        platform.process.set_output(
            "",
            CommandOutput {
                status: Some(0),
                stdout: metadata.as_bytes().to_vec(),
                ..CommandOutput::default()
            },
        );
        platform
    }

    fn target(kind: TargetKind, root: &str, externs: &[&str]) -> Target {
        Target {
            kind,
            root: PathBuf::from(root),
            externs: externs.iter().map(ToString::to_string).collect(),
        }
    }

    mod packages {
        use super::*;

        #[test]
        fn packages_should_list_the_targets_their_externs_and_the_features() {
            let platform = platform(
                r#"{"packages": [{
                    "name": "my-crate",
                    "manifest_path": "/c/Cargo.toml",
                    "targets": [
                        {"name": "build-script-build", "kind": ["custom-build"], "src_path": "/c/build.rs"},
                        {"name": "it", "kind": ["test"], "src_path": "/c/tests/it.rs"},
                        {"name": "my-crate", "kind": ["lib"], "src_path": "/c/src/lib.rs"},
                        {"name": "tool", "kind": ["bin"], "src_path": "/c/src/main.rs"},
                        {"name": "e", "kind": ["example"], "src_path": "/c/examples/e.rs"},
                        {"name": "b", "kind": ["bench"], "src_path": "/c/benches/b.rs"}
                    ],
                    "dependencies": [
                        {"name": "serde-json", "rename": null, "kind": null},
                        {"name": "real", "rename": "alias", "kind": null},
                        {"name": "dev", "rename": null, "kind": "dev"},
                        {"name": "cc", "rename": null, "kind": "build"}
                    ],
                    "features": {"serde": ["dep:serde"], "default": ["serde"]}
                }]}"#,
            );

            let packages = packages(&platform, Path::new("/c/Cargo.toml")).unwrap();

            let deps = ["serde_json", "alias", "dev"];
            let with_lib = ["serde_json", "alias", "dev", "my_crate"];
            assert_eq!(
                packages,
                vec![Package {
                    name: "my-crate".to_string(),
                    manifest: PathBuf::from("/c/Cargo.toml"),
                    targets: vec![
                        target(TargetKind::Lib, "/c/src/lib.rs", &deps),
                        target(TargetKind::Bin, "/c/src/main.rs", &with_lib),
                        target(TargetKind::Test, "/c/tests/it.rs", &with_lib),
                        target(TargetKind::Example, "/c/examples/e.rs", &with_lib),
                        target(TargetKind::Bench, "/c/benches/b.rs", &with_lib),
                        target(TargetKind::Build, "/c/build.rs", &["cc", "my_crate"]),
                    ],
                    features: vec!["default".to_string(), "serde".to_string()],
                }]
            );
            assert_eq!(
                platform.process.calls(),
                vec!["cargo metadata --no-deps --format-version 1 --manifest-path /c/Cargo.toml"]
            );
        }

        #[test]
        fn packages_proc_macro_or_without_library_should_name_no_package_crate() {
            let platform = platform(
                r#"{"packages": [
                    {"name": "derive", "manifest_path": "/d/Cargo.toml", "dependencies": [], "targets": [
                        {"name": "derive", "kind": ["proc-macro"], "src_path": "/d/src/lib.rs"},
                        {"name": "t", "kind": ["test"], "src_path": "/d/tests/t.rs"}
                    ]},
                    {"name": "app", "manifest_path": "/a/Cargo.toml", "dependencies": [], "targets": [
                        {"name": "app", "kind": ["bin"], "src_path": "/a/src/main.rs"}
                    ]}
                ]}"#,
            );

            let packages = packages(&platform, Path::new("/Cargo.toml")).unwrap();

            assert_eq!(
                packages[0].targets,
                vec![
                    target(TargetKind::Lib, "/d/src/lib.rs", &["proc_macro"]),
                    target(TargetKind::Test, "/d/tests/t.rs", &["derive"]),
                ]
            );
            assert_eq!(
                packages[1].targets,
                vec![target(TargetKind::Bin, "/a/src/main.rs", &[])]
            );
        }

        #[test]
        fn packages_should_run_the_cargo_of_the_environment() {
            let platform = platform(r#"{"packages": []}"#)
                .with_env(TestEnv::new().with_vars(vec![("CARGO", "/bin/my-cargo")]));

            let packages = packages(&platform, Path::new("/c/Cargo.toml")).unwrap();

            assert_eq!(packages, vec![]);
            assert!(
                platform.process.calls()[0].starts_with("/bin/my-cargo metadata"),
                "{:?}",
                platform.process.calls()
            );
        }

        #[test]
        fn packages_failing_cargo_should_be_reported() {
            let failing = test_platform();
            failing.process.set_output(
                "cargo metadata",
                CommandOutput {
                    status: Some(101),
                    stderr: b"error: failed to parse manifest\n".to_vec(),
                    ..CommandOutput::default()
                },
            );
            let missing = test_platform();
            missing
                .process
                .set_io_error("cargo", std::io::ErrorKind::NotFound);
            let invalid = platform("{");

            let errors = [&failing, &missing, &invalid]
                .map(|platform| packages(platform, Path::new("/c/Cargo.toml")).unwrap_err());

            // WHY: paths display with `\` on Windows.
            let errors = errors.map(|e| e.to_string().replace('\\', "/"));
            assert_eq!(
                errors[0],
                "/c/Cargo.toml: cargo metadata failed: error: failed to parse manifest"
            );
            assert!(
                errors[1].starts_with("/c/Cargo.toml: cargo metadata failed: "),
                "{}",
                errors[1]
            );
            assert!(
                errors[2].starts_with("/c/Cargo.toml: cargo metadata failed: EOF"),
                "{}",
                errors[2]
            );
        }
    }

    mod target {
        use super::*;

        mod new {
            use super::*;

            #[test]
            fn new_should_have_no_externs() {
                assert_eq!(
                    Target::new(TargetKind::Lib, "/c/lib.rs"),
                    target(TargetKind::Lib, "/c/lib.rs", &[])
                );
            }
        }
    }
}
