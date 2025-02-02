use crate::utilities;
use crate::CIError;
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
