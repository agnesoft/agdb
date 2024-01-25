use std::io::BufRead;

const RUST_RELEASE_PROJECTS: [&str; 4] = ["agdb", "agdb_derive", "agdb_api", "agdb_server"];
const AGDB_PROJECT: &str = "agdb";
const CARGO_TOML: &str = "Cargo.toml";
const PACKAGE_JSON: &str = "package.json";
const IGNORE: [&str; 8] = [
    "node_modules",
    "tests",
    "target",
    "src",
    "dist",
    ".nuxt",
    "playwright-report",
    "test-results",
];
const TYPESCRIPT_PROJECTS: [&str; 1] = ["@agnesoft/agdb_api"];

#[allow(dead_code)]
#[derive(Debug)]
struct CIError {
    description: String,
}

#[derive(Default)]
struct ProjectFiles {
    cargo_tomls: Vec<std::path::PathBuf>,
    package_jsons: Vec<std::path::PathBuf>,
}

impl<E: std::error::Error> From<E> for CIError {
    fn from(error: E) -> Self {
        Self {
            description: format!("{:?}", error),
        }
    }
}

fn current_version() -> Result<String, CIError> {
    let cargo_toml = std::path::Path::new(AGDB_PROJECT).join(CARGO_TOML);
    std::io::BufReader::new(std::fs::File::open(cargo_toml)?)
        .lines()
        .find_map(|line| {
            line.ok().and_then(|line| {
                if line.starts_with("version = ") {
                    return Some(line.split('"').nth(1).unwrap_or("").to_string());
                }

                None
            })
        })
        .ok_or(CIError {
            description: "Current version not found".to_string(),
        })
}

fn project_files(path: &std::path::Path, files: &mut ProjectFiles) -> Result<(), CIError> {
    for dir in std::fs::read_dir(path)? {
        let dir = dir?;

        if dir.file_type()?.is_dir()
            && !IGNORE.contains(&dir.file_name().to_string_lossy().as_ref())
        {
            if dir.path().join(CARGO_TOML).exists() {
                files.cargo_tomls.push(dir.path().join(CARGO_TOML));
            } else if dir.path().join(PACKAGE_JSON).exists() {
                files.package_jsons.push(dir.path().join(PACKAGE_JSON));
            }

            project_files(&dir.path(), files)?;
        }
    }

    Ok(())
}

fn update_cargo_projects(
    current_version: &str,
    new_version: &str,
    cargo_tomls: &[std::path::PathBuf],
) -> Result<(), CIError> {
    for cargo_toml in cargo_tomls {
        println!("Updating... {:?}", cargo_toml);

        let mut content = std::fs::read_to_string(cargo_toml)?.replace(
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

    Ok(())
}

fn update_typescript_projects(
    current_version: &str,
    new_version: &str,
    package_jsons: &[std::path::PathBuf],
) -> Result<(), CIError> {
    for package_json in package_jsons {
        println!("Updating... {:?}", package_json);

        let mut content = std::fs::read_to_string(package_json)?.replace(
            &format!("\"version\": \"{current_version}\""),
            &format!("\"version\": \"{new_version}\""),
        );

        for project in TYPESCRIPT_PROJECTS {
            content = content
                .replace(
                    &format!("\"{project}\": \"{current_version}\""),
                    &format!("\"{project}\": \"{new_version}\""),
                )
                .replace(
                    &format!("\"{project}\": \"^{current_version}\""),
                    &format!("\"{project}\": \"^{new_version}\""),
                );
        }

        std::fs::write(package_json, content)?;
        std::process::Command::new("bash")
            .arg("-c")
            .arg("npm install")
            .current_dir(package_json.parent().expect("Parent directory not found"))
            .spawn()?
            .wait()?;
    }
    Ok(())
}

fn main() -> Result<(), CIError> {
    let current_version = current_version()?;
    let new_version = std::fs::read_to_string(std::path::Path::new("Version"))?
        .trim()
        .to_string();

    println!("Current version: {}", current_version);
    println!("New version: {}", new_version);

    let mut files = ProjectFiles::default();
    project_files(std::path::Path::new("."), &mut files)?;
    update_cargo_projects(&current_version, &new_version, &files.cargo_tomls)?;
    update_typescript_projects(&current_version, &new_version, &files.package_jsons)?;

    println!("DONE");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct MyError {}
    impl std::error::Error for MyError {}
    impl std::fmt::Display for MyError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "MyError")
        }
    }

    #[test]
    fn derived_from_debug() {
        let error = CIError::from(MyError {});

        assert_eq!(format!("{:?}", MyError {}), "MyError");
        assert_eq!(format!("{}", MyError {}), "MyError");
        assert_eq!(
            format!("{:?}", error),
            "CIError { description: \"MyError\" }"
        );
    }
}
