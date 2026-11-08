use crate::CIError;
use crate::utilities;
use std::path::Path;
use std::process::Command;

pub(crate) fn generate_test_queries() -> Result<(), CIError> {
    println!("Generating PHP test_queries");
    utilities::run_command(
        Command::new(utilities::BASH)
            .arg("ci.sh")
            .arg("test_queries")
            .current_dir(Path::new("agdb_api").join("php")),
    )?;
    Ok(())
}

pub(crate) fn generate_api() -> Result<(), CIError> {
    println!("Generating PHP openapi");
    utilities::run_command(
        Command::new(utilities::BASH)
            .arg("ci.sh")
            .arg("openapi")
            .current_dir(Path::new("agdb_api").join("php")),
    )?;
    Ok(())
}

#[derive(serde::Deserialize)]
pub(crate) struct Autoload {
    pub classmap: Vec<String>,
}

#[derive(serde::Deserialize)]
pub(crate) struct Composer {
    pub name: String,
    pub autoload: Autoload,
}

pub(crate) struct Package {
    pub name: String,
    pub path: String,
}

pub(crate) fn packages() -> Result<Vec<Package>, CIError> {
    #[cfg(windows)]
    let output = Command::new("cmd")
        .arg("/C")
        .arg("composer show --self --format=json")
        .output()?;
    #[cfg(not(windows))]
    let output = Command::new("composer")
        .arg("show")
        .arg("--self")
        .arg("--format=json")
        .output()?;
    let composer: Composer = serde_json::from_slice(&output.stdout)?;

    Ok(composer
        .autoload
        .classmap
        .first()
        .map(|path| Package {
            name: composer.name.clone(),
            path: path
                .trim()
                .trim_end_matches("src")
                .trim_end_matches("lib")
                .into(),
        })
        .into_iter()
        .collect::<Vec<Package>>())
}
