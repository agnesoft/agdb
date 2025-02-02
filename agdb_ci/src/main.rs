mod ci_error;
mod languages;
mod sources;
mod utilities;

use crate::ci_error::CIError;
use crate::languages::php;
use crate::languages::rust;
use crate::languages::typescript;
use std::path::Path;

fn ci() -> Result<(), CIError> {
    let current_version = sources::current_version()?;
    let new_version = sources::new_version()?;
    println!("Current version: {}", current_version);
    println!("New version: {}", new_version);

    languages::update_versions(Path::new("./"), &current_version, &new_version)?;

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
