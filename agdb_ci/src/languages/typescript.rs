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

    let project_dir = json.parent().expect("Parent directory not found");
    println!(
        "Installing dependencies in '{}'",
        project_dir.to_string_lossy()
    );
    utilities::run_command(
        Command::new(utilities::BASH)
            .arg("-c")
            .arg("npm install")
            .current_dir(project_dir),
    )?;
    let _ = utilities::run_command(
        Command::new(utilities::BASH)
            .arg("-c")
            .arg("npm audit fix")
            .current_dir(project_dir),
    );

    Ok(())
}

pub(crate) fn generate_test_queries() -> Result<(), CIError> {
    println!("Generating Typescript test_queries");
    utilities::run_command(
        Command::new(utilities::BASH)
            .arg("-c")
            .arg("npm run test_queries")
            .current_dir(Path::new("agdb_api").join("typescript")),
    )?;
    Ok(())
}

pub(crate) fn generate_api() -> Result<(), CIError> {
    println!("Generating Typescript openapi");
    utilities::run_command(
        Command::new(utilities::BASH)
            .arg("-c")
            .arg("npm run openapi")
            .current_dir(Path::new("agdb_api").join("typescript")),
    )?;
    Ok(())
}
