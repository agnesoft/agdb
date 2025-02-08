pub(crate) mod php;
pub(crate) mod rust;
pub(crate) mod typescript;

use crate::CIError;
use std::path::Path;

const IGNORE: [&str; 10] = [
    "node_modules",
    "vendor",
    "tests",
    "target",
    "src",
    "dist",
    "playwright-report",
    "test-results",
    ".openapi-generator",
    "coverage",
];

pub(crate) fn update_versions(
    path: &Path,
    current_version: &str,
    new_version: &str,
) -> Result<(), CIError> {
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        let filename = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        if IGNORE.contains(&filename.as_str()) {
            continue;
        }

        if path.is_dir() {
            update_versions(path.as_path(), current_version, new_version)?;
        } else if filename.ends_with(".toml") {
            println!("Updating '{}'", path.to_string_lossy().replace('\\', "/"));
            rust::update_version(path.as_path(), current_version, new_version)?;
        } else if filename == "package.json" {
            println!("Updating '{}'", path.to_string_lossy().replace('\\', "/"));
            typescript::update_version(path.as_path(), current_version, new_version)?;
        }
    }

    Ok(())
}

trait Language {
    fn generate_type(ty: &agdb::api::Type) -> String;
    fn type_name(ty: &agdb::api::Type) -> String;
}
