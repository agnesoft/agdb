use crate::utilities::run_command;
use crate::CIError;
use std::path::Path;
use std::process::Command;

pub(crate) fn current_version() -> Result<String, CIError> {
    Ok(
        run_command(Command::new("git").arg("tag").arg("--sort=taggerdate"))?
            .trim()
            .lines()
            .last()
            .ok_or("tags not found")?
            .to_string(),
    )
}

pub(crate) fn new_version() -> Result<String, CIError> {
    Ok(std::fs::read_to_string(Path::new("Version"))?
        .trim()
        .to_string())
}
