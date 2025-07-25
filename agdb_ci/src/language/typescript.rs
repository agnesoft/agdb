use crate::CIError;
use crate::utilities;
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
