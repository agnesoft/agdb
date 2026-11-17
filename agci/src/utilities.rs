use crate::CIError;
use agplatform::CommandExt;
use agplatform::CommandOptions;
use agplatform::CommandOutput;
use agplatform::Env;
use agplatform::Fs;
use agplatform::OpenWriteOptions;
use agplatform::OutputStream;
use agplatform::Platform;
use agplatform::Process;
use agplatform::ProcessCommand;
use std::io::Write;
use std::path::Path;
use std::process::Command;

#[cfg(target_os = "windows")]
pub(crate) const BASH: &str = "C:/Program Files/Git/bin/bash.exe";
#[cfg(not(target_os = "windows"))]
pub(crate) const BASH: &str = "bash";

pub(crate) fn run_command(platform: &impl Platform, command: &Command) -> Result<String, CIError> {
    let out = output(platform, command)?;
    std::io::stdout().write_all(&out.stdout)?;
    std::io::stderr().write_all(&out.stderr)?;
    if out.success {
        Ok(String::from_utf8(out.stdout)?)
    } else {
        Err(failed(command))
    }
}

/// Runs the command with its stdout and stderr both streamed to `out`, so that the output stays
/// in order (and inside the GitHub log group of the check) as one stream.
pub(crate) fn stream_command(
    platform: &impl Platform,
    command: &Command,
    out: &mut (dyn Write + Send),
) -> Result<(), CIError> {
    out.flush()?;
    let result =
        command.run_blocking_with_sink(platform.process(), &mut |_: OutputStream, chunk: &[u8]| {
            out.write_all(chunk)
                .and_then(|()| out.flush())
                .map_err(agplatform::Error::other)
        });

    match result {
        Ok(_) => Ok(()),
        Err(e) if e.command_output().is_some() => Err(failed(command)),
        Err(e) => Err(e.into()),
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
pub(crate) fn run_quiet(platform: &impl Platform, command: &Command) -> Result<String, CIError> {
    quiet(output(platform, command)?, command)
}

/// [`run_quiet`] with `input` written to the standard input of the command.
pub(crate) fn run_quiet_with_input(
    platform: &impl Platform,
    command: &Command,
    input: String,
) -> Result<String, CIError> {
    let options = CommandOptions {
        input: Some(input.into_bytes()),
        ..Default::default()
    };
    let out = platform
        .process()
        .run_blocking(ProcessCommand::with_options(command, options));
    quiet(output_of(out)?, command)
}

fn quiet(out: CommandOutput, command: &Command) -> Result<String, CIError> {
    if out.success {
        Ok(String::from_utf8(out.stdout)?)
    } else {
        std::io::stderr().write_all(&out.stderr)?;
        Err(format!("Command failed: {command:?}").into())
    }
}

/// The output of the command that ran, whether it succeeded or not.
fn output(platform: &impl Platform, command: &Command) -> Result<CommandOutput, CIError> {
    output_of(command.run_blocking(platform.process()))
}

fn output_of(result: agplatform::Result<CommandOutput>) -> Result<CommandOutput, CIError> {
    match result {
        Ok(out) => Ok(out),
        Err(e) => match e.command_output() {
            Some(out) => Ok(out.clone()),
            None => Err(e.into()),
        },
    }
}

/// None if the file does not exist.
pub(crate) fn read_if_exists(
    platform: &impl Platform,
    path: &str,
) -> Result<Option<String>, CIError> {
    match platform.fs().read_to_string(path) {
        Ok(content) => Ok(Some(content)),
        Err(e)
            if e.source_as::<std::io::Error>()
                .is_some_and(|e| e.kind() == std::io::ErrorKind::NotFound) =>
        {
            Ok(None)
        }
        Err(e) => Err(e.into()),
    }
}

/// Appends `name=value` to the GitHub step outputs, a no-op outside of GitHub Actions.
pub(crate) fn set_output(platform: &impl Platform, name: &str, value: &str) -> Result<(), CIError> {
    if let Some(path) = platform.env().var("GITHUB_OUTPUT") {
        writeln!(
            platform
                .fs()
                .open_write_options(path, OpenWriteOptions::new().append())?,
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
