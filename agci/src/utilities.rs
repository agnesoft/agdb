use crate::CIError;
use std::io::Write;
use std::path::Path;
use std::process::Command;

#[cfg(target_os = "windows")]
pub(crate) const BASH: &str = "C:/Program Files/Git/bin/bash.exe";
#[cfg(not(target_os = "windows"))]
pub(crate) const BASH: &str = "bash";

pub(crate) fn run_command(command: &mut Command) -> Result<String, CIError> {
    let out = command.output()?;
    std::io::stdout().write_all(&out.stdout)?;
    std::io::stderr().write_all(&out.stderr)?;
    if out.status.success() {
        Ok(String::from_utf8(out.stdout)?)
    } else {
        Err(format!(
            "Command failed: {command:?} ({})",
            command
                .get_current_dir()
                .unwrap_or(Path::new("."))
                .to_string_lossy()
        )
        .into())
    }
}
