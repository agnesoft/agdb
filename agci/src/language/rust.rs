use crate::CIError;
use crate::utilities;
use std::process::Command;

pub(crate) fn generate_test_queries() -> Result<(), CIError> {
    println!("Generating test_queries.json");
    utilities::run_command(
        Command::new("cargo")
            .arg("test")
            .arg("-p")
            .arg("agdb_server")
            .arg("tests::test_queries")
            .arg("--")
            .arg("--exact"),
    )?;
    Ok(())
}

pub(crate) fn generate_api() -> Result<(), CIError> {
    println!("Generating openapi.json");
    utilities::run_command(
        Command::new("cargo")
            .arg("test")
            .arg("-p")
            .arg("agdb_server")
            .arg("api::tests::openapi")
            .arg("--")
            .arg("--exact"),
    )?;
    Ok(())
}

#[derive(serde::Deserialize)]
pub(crate) struct Metadata {
    pub packages: Vec<Package>,
}

#[derive(serde::Deserialize)]
pub(crate) struct Dependency {
    pub name: String,
    pub req: String,
    pub path: Option<String>,
}

#[derive(serde::Deserialize)]
pub(crate) struct Package {
    pub name: String,
    pub manifest_path: String,
    #[serde(default)]
    pub dependencies: Vec<Dependency>,
    #[serde(default)]
    pub dev_dependencies: Vec<Dependency>,
}

pub(crate) fn packages() -> Result<Vec<Package>, CIError> {
    let output = Command::new("cargo")
        .arg("metadata")
        .arg("--no-deps")
        .output()?;
    let metadata: Metadata = serde_json::from_slice(&output.stdout)?;
    Ok(metadata
        .packages
        .into_iter()
        .filter(|p| !p.manifest_path.is_empty())
        .collect())
}
