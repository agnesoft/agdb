use crate::CIError;
use crate::project_type::Registry;
use crate::utilities;
use std::process::Command;

pub(super) struct CratesIo;

impl Registry for CratesIo {
    fn name(&self) -> &'static str {
        "crates.io"
    }

    fn published(&self, package: &str, version: &str) -> Result<bool, CIError> {
        let status = utilities::run_quiet(Command::new("curl").args([
            "-s",
            "-o",
            "/dev/null",
            "-w",
            "%{http_code}",
            "-A",
            "agci (https://github.com/agnesoft/agdb)",
            &format!("https://crates.io/api/v1/crates/{package}/{version}"),
        ]))?;

        match status.trim() {
            "200" => Ok(true),
            "404" => Ok(false),
            s => Err(format!("crates.io responded {s} for {package}@{version}").into()),
        }
    }
}
