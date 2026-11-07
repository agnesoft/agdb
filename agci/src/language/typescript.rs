use crate::CIError;
use crate::utilities;
use std::collections::BTreeMap;
use std::path::Path;
use std::process::Command;

pub(crate) fn update_version(
    json: &Path,
    current_version: &str,
    new_version: &str,
) -> Result<(), CIError> {
    let content = std::fs::read_to_string(json)?.replace(
        &format!("\"version\": \"{current_version}\""),
        &format!("\"version\": \"{new_version}\""),
    );
    std::fs::write(json, content)?;
    Ok(())
}

pub(crate) fn generate_test_queries() -> Result<(), CIError> {
    println!("Generating Typescript test_queries");
    utilities::run_command(
        Command::new(utilities::BASH)
            .arg("-c")
            .arg("pnpm run test_queries")
            .current_dir(Path::new("agdb_api").join("typescript")),
    )?;
    Ok(())
}

pub(crate) fn generate_api() -> Result<(), CIError> {
    println!("Generating Typescript openapi");
    utilities::run_command(
        Command::new(utilities::BASH)
            .arg("-c")
            .arg("pnpm run openapi")
            .current_dir(Path::new("agdb_api").join("typescript")),
    )?;
    Ok(())
}

#[derive(serde::Deserialize)]
pub(crate) struct Dependency {
    pub from: String,
    pub version: String,
    pub path: String,
}

#[derive(serde::Deserialize)]
pub(crate) struct Package {
    pub path: String,
    pub private: bool,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub dependencies: BTreeMap<String, Dependency>,
    #[serde(default)]
    pub dev_dependencies: BTreeMap<String, Dependency>,
}

pub(crate) fn packages() -> Result<Vec<Package>, CIError> {
    let output = Command::new("bash")
        .arg("-c")
        .arg("pnpm list --only-projects --json")
        .output()?;
    Ok(serde_json::from_slice(&output.stdout)?)
}
