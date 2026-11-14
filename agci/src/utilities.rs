use crate::CIError;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::process::Stdio;

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
        Err(failed(command))
    }
}

/// Runs the command with its stdout and stderr both streamed to stdout, so that the output stays
/// in order (and inside the GitHub log group of the check) as one stream.
pub(crate) fn stream_command(command: &mut Command) -> Result<(), CIError> {
    std::io::stdout().flush()?;
    let status = command
        .stdout(Stdio::inherit())
        .stderr(Stdio::from(std::io::stdout()))
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(failed(command))
    }
}

fn failed(command: &Command) -> CIError {
    format!(
        "Command failed: {command:?} ({})",
        command
            .get_current_dir()
            .unwrap_or(Path::new("."))
            .to_string_lossy()
    )
    .into()
}

/// Runs the command without echoing its output, returning stdout.
pub(crate) fn run_quiet(command: &mut Command) -> Result<String, CIError> {
    let out = command.output()?;
    if out.status.success() {
        Ok(String::from_utf8(out.stdout)?)
    } else {
        std::io::stderr().write_all(&out.stderr)?;
        Err(format!("Command failed: {command:?}").into())
    }
}

/// Appends `name=value` to the GitHub step outputs, a no-op outside of GitHub Actions.
pub(crate) fn set_output(name: &str, value: &str) -> Result<(), CIError> {
    if let Ok(path) = std::env::var("GITHUB_OUTPUT") {
        writeln!(
            std::fs::OpenOptions::new().append(true).open(path)?,
            "{name}={value}"
        )?;
    }

    Ok(())
}

/// FNV-1a: stable across Rust versions and platforms (unlike `DefaultHasher`), so the hashes can
/// be compared across runs.
pub(crate) fn hash(data: &[u8]) -> String {
    let hash = data.iter().fold(0xcbf2_9ce4_8422_2325_u64, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
    });
    format!("{hash:016x}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_is_fnv1a() {
        assert_eq!(hash(b""), "cbf29ce484222325");
        assert_eq!(hash(b"a"), "af63dc4c8601ec8c");
    }
}
