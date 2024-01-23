use std::io::BufRead;

const RUST_RELEASE_PROJECTS: [&str; 4] = ["agdb", "agdb_derive", "agdb_server", "agdb_api/rust"];
const TYPESCRIPT_PROJECTS: [&str; 1] = ["agdb_api/typescript"];
const CARGO_TOML: &str = "Cargo.toml";
//const PACKAGE_JSON: &str = "package.json";

#[allow(dead_code)]
#[derive(Debug)]
struct CIError {
    description: String,
}

impl<E: std::error::Error> From<E> for CIError {
    fn from(error: E) -> Self {
        Self {
            description: error.to_string(),
        }
    }
}

fn current_version() -> Result<String, CIError> {
    let cargo_toml = std::path::Path::new(RUST_RELEASE_PROJECTS[0]).join(CARGO_TOML);
    std::io::BufReader::new(std::fs::File::open(cargo_toml)?)
        .lines()
        .find_map(|line| {
            if let Ok(line) = line {
                if line.starts_with("version = ") {
                    return Some(line.split('"').nth(1).unwrap_or("").to_string());
                }
            }

            None
        })
        .ok_or(CIError {
            description: "Current version not found".to_string(),
        })
}

fn cargo_tomls(path: &std::path::Path) -> Result<Vec<std::path::PathBuf>, CIError> {
    let mut tomls = Vec::new();

    for dir in std::fs::read_dir(path)? {
        let dir = dir?;

        if dir.file_type()?.is_dir() {
            let cargo_toml = dir.path().join(CARGO_TOML);

            if cargo_toml.exists() {
                tomls.push(cargo_toml);
            }

            tomls.extend(cargo_tomls(&dir.path())?);
        }
    }

    Ok(tomls)
}

fn main() -> Result<(), CIError> {
    let current_version = current_version()?;
    let new_version = std::fs::read_to_string(std::path::Path::new("Version"))?;
    let cargo_tomls = cargo_tomls(std::path::Path::new("."))?;

    println!("Current version: {}", current_version);
    println!("New version: {}", new_version);

    if current_version != new_version {
        for cargo_toml in cargo_tomls {
            let mut content = std::fs::read_to_string(&cargo_toml)?.replace(
                &format!("\nversion = \"{current_version}\""),
                &format!("\nversion = \"{new_version}\""),
            );

            for project in RUST_RELEASE_PROJECTS {
                content = content
                    .replace(
                        &format!("{project} = \"{current_version}\""),
                        &format!("{project} = \"{new_version}\""),
                    )
                    .replace(
                        &format!("{project} = {{ version = \"{current_version}\""),
                        &format!("{project} = {{ version = \"{new_version}\""),
                    );
            }

            std::fs::write(cargo_toml, content)?;
        }
    }

    Ok(())
}
