use crate::CIError;
use crate::utilities::run_command;
use std::path::Path;
use std::process::Command;

pub(crate) fn current_version() -> Result<String, CIError> {
    Ok(
        run_command(Command::new("git").arg("tag").arg("--sort=taggerdate"))?
            .trim()
            .lines()
            .last()
            .ok_or("tags not found")?[1..]
            .to_string(),
    )
}

pub(crate) fn new_version() -> Result<String, CIError> {
    Ok(std::fs::read_to_string(Path::new("Cargo.toml"))?
        .lines()
        .find(|line| line.trim_start().starts_with("version"))
        .and_then(|line| line.trim_start().split('"').nth(1).map(|v| v.to_string()))
        .ok_or("version not found")?)
}
