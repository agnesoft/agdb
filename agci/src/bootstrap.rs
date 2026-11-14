use crate::CIError;
use crate::affected::RUST_INPUTS;
use crate::affected::graph;
use crate::project::Project;
use crate::utilities;
use agdb::QueryBuilder;
use std::collections::BTreeSet;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

const PACKAGE: &str = "agci";
const EXE: &str = if cfg!(windows) { "agci.exe" } else { "agci" };
const FINGERPRINT: &str = "agci.fingerprint";

/// Repository paths the agci binary is built from: its project, the projects it transitively depends
/// on and the files affecting all Rust projects. Extra inputs (`depends` paths) are not compiled in.
pub(crate) fn inputs(projects: &[Project]) -> Result<BTreeSet<String>, CIError> {
    let agci = projects
        .iter()
        .find(|p| p.packages.iter().any(|name| name == PACKAGE))
        .ok_or("the agci project not found")?;
    let closure = graph(projects)?.exec(
        QueryBuilder::select()
            .aliases()
            .search()
            .from(agci.dir.as_str())
            .where_()
            .node()
            .query(),
    )?;
    let mut paths: BTreeSet<String> = RUST_INPUTS
        .iter()
        .map(|input| input.trim_end_matches('/').to_string())
        .collect();

    for element in &closure.elements {
        paths.insert(element.values[0].value.string()?.clone());
    }

    Ok(paths)
}

/// The git objects of the inputs on HEAD (missing ones are left out), one `<mode> <type> <hash>\t<path>`
/// line each, so a change of any of them changes it.
fn fingerprint(projects: &[Project]) -> Result<String, CIError> {
    let mut args = vec!["ls-tree".to_string(), "HEAD".to_string(), "--".to_string()];
    args.extend(inputs(projects)?);
    utilities::run_quiet(Command::new("git").args(&args))
}

/// Whether the binary in `dir` is outdated: the fingerprint it was built with differs from the current
/// one. Written as `rebuild=<bool>` to the GitHub step outputs. Any error (e.g. the binary cannot
/// parse metadata added after it was built) means it is outdated.
pub(crate) fn check(dir: &Path, projects: Result<Vec<Project>, CIError>) -> Result<(), CIError> {
    let built = std::fs::read_to_string(dir.join(FINGERPRINT)).unwrap_or_default();
    let rebuild = match projects.and_then(|projects| fingerprint(&projects)) {
        Ok(current) if current == built => {
            println!("agci in '{}' is up to date", dir.display());
            false
        }
        Ok(current) => {
            let built: BTreeSet<&str> = built.lines().collect();
            println!("agci in '{}' is outdated, changed:", dir.display());

            for line in current.lines().filter(|line| !built.contains(line)) {
                println!("  {line}");
            }

            true
        }
        Err(e) => {
            println!("agci in '{}' is outdated: {}", dir.display(), e.description);
            true
        }
    };

    utilities::set_output("rebuild", &rebuild.to_string())
}

/// Builds agci from the sources into `dir` along with its fingerprint.
pub(crate) fn build(dir: &Path, projects: &[Project]) -> Result<(), CIError> {
    let fingerprint = fingerprint(projects)?;
    utilities::stream_command(Command::new("cargo").args(["build", "-r", "-p", PACKAGE]))?;
    std::fs::create_dir_all(dir)?;
    std::fs::copy(target_dir()?.join("release").join(EXE), dir.join(EXE))?;
    std::fs::write(dir.join(FINGERPRINT), fingerprint)?;
    println!("agci built into '{}'", dir.display());
    Ok(())
}

fn target_dir() -> Result<PathBuf, CIError> {
    let metadata: serde_json::Value =
        serde_json::from_str(&utilities::run_quiet(Command::new("cargo").args([
            "metadata",
            "--no-deps",
            "--format-version",
            "1",
        ]))?)?;
    Ok(PathBuf::from(
        metadata["target_directory"]
            .as_str()
            .ok_or("cargo metadata without target_directory")?,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::Lang;
    use crate::project::tests::config;

    fn project(dir: &str, package: &str, depends: &[&str], inputs: &[&str]) -> Project {
        Project {
            dir: dir.to_string(),
            lang: Lang::Rust,
            packages: vec![package.to_string()],
            config: config(&[], &[]),
            depends: depends.iter().map(|d| d.to_string()).collect(),
            inputs: inputs.iter().map(|i| i.to_string()).collect(),
        }
    }

    #[test]
    fn inputs_are_the_transitive_dependencies_and_rust_inputs_but_not_extra_inputs() {
        let projects = vec![
            project("agci", "agci", &["db"], &[]),
            project("db", "agdb", &["derive"], &["schema/db.json"]),
            project("derive", "agdb_derive", &[], &[]),
            project("server", "agdb_server", &["db"], &[]),
        ];

        assert_eq!(
            inputs(&projects).unwrap(),
            BTreeSet::from(
                [
                    ".cargo",
                    "Cargo.lock",
                    "Cargo.toml",
                    "agci",
                    "db",
                    "derive",
                    "rust-toolchain",
                    "rust-toolchain.toml",
                ]
                .map(str::to_string)
            )
        );
    }

    #[test]
    fn inputs_without_agci_fail() {
        let projects = vec![project("db", "agdb", &[], &[])];

        assert_eq!(
            inputs(&projects).unwrap_err().description,
            "the agci project not found"
        );
    }
}
