mod ci_error;
mod language;
mod sources;
mod utilities;

use crate::ci_error::CIError;
use crate::language::php;
use crate::language::rust;
use crate::language::typescript;
use std::path::Path;
use std::process::Command;

fn ci() -> Result<(), CIError> {
    let current_version = sources::current_version()?;
    let new_version = sources::new_version()?;
    println!("Current version: {current_version}");
    println!("New version: {new_version}");
    language::update_versions(Path::new("./"), &current_version, &new_version)?;

    println!("Installing global dependencies");
    utilities::run_command(Command::new(utilities::BASH).arg("-c").arg("pnpm i"))?;

    rust::generate_api()?;
    typescript::generate_api()?;
    php::generate_api()?;

    rust::generate_test_queries()?;
    typescript::generate_test_queries()?;
    php::generate_test_queries()?;

    println!("DONE");

    Ok(())
}

fn main() -> Result<(), CIError> {
    ci().inspect_err(|e| println!("Error: {}", e.description))
}
