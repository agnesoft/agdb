use crate::CIError;
use crate::project_type::Registry;
use std::process::Command;

pub(super) struct Npm;

impl Registry for Npm {
    fn name(&self) -> &'static str {
        "npm"
    }

    /// `npm view` prints nothing for an unknown version and fails with E404 for an unknown package.
    fn published(&self, package: &str, version: &str) -> Result<bool, CIError> {
        let npm = if cfg!(windows) { "npm.cmd" } else { "npm" };
        let out = Command::new(npm)
            .args(["view", &format!("{package}@{version}"), "version"])
            .output()?;
        let stderr = String::from_utf8_lossy(&out.stderr);

        if out.status.success() {
            Ok(!out.stdout.trim_ascii().is_empty())
        } else if stderr.contains("E404") {
            Ok(false)
        } else {
            Err(format!("npm view {package}@{version} failed: {stderr}").into())
        }
    }
}
