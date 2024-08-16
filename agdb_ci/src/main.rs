use std::io::BufRead;

const RUST_RELEASE_PROJECTS: [&str; 4] = ["agdb", "agdb_derive", "agdb_api", "agdb_server"];
const AGDB_PROJECT: &str = "agdb";
const CARGO_TOML: &str = "Cargo.toml";
const PACKAGE_JSON: &str = "package.json";
const OPENAPI_JSON: &str = "schema.json";
const API_MD: &str = "openapi.en-US.mdx";
const IGNORE: [&str; 9] = [
    "node_modules",
    "vendor",
    "tests",
    "target",
    "src",
    "dist",
    ".nuxt",
    "playwright-report",
    "test-results",
];
const TYPESCRIPT_PROJECTS: [&str; 1] = ["@agnesoft/agdb_api"];
#[cfg(windows)]
const LN: &str = "\r\n";
#[cfg(not(windows))]
const LN: &str = "\n";

#[derive(Debug)]
struct CIError {
    #[allow(dead_code)]
    description: String,
}

#[derive(Default)]
struct ProjectFiles {
    tomls: Vec<std::path::PathBuf>,
    jsons: Vec<std::path::PathBuf>,
    schema: std::path::PathBuf,
    api: std::path::PathBuf,
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
                files.tomls.push(dir.path().join(CARGO_TOML));
            } else if dir.path().join(PACKAGE_JSON).exists() {
                files.jsons.push(dir.path().join(PACKAGE_JSON));
            } else if dir.path().join(OPENAPI_JSON).exists() {
                files.jsons.push(dir.path().join(OPENAPI_JSON));
                files.schema = dir.path().join(OPENAPI_JSON);
            } else if dir.path().join(API_MD).exists() {
                files.api = dir.path().join(API_MD);
            }

            project_files(&dir.path(), files)?;
        }
    }

    Ok(())
}

fn update_tomls(
    current_version: &str,
    new_version: &str,
    tomls: &[std::path::PathBuf],
) -> Result<(), CIError> {
    for toml in tomls {
        println!("Updating... {:?}", toml);

        let mut content = std::fs::read_to_string(toml)?.replace(
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

        std::fs::write(toml, content)?;
    }

    Ok(())
}

fn update_jsons(
    current_version: &str,
    new_version: &str,
    jsons: &[std::path::PathBuf],
) -> Result<(), CIError> {
    for json in jsons {
        println!("Updating... {:?}", json);

        let mut content = std::fs::read_to_string(json)?.replace(
            &format!("\"version\": \"{current_version}\""),
            &format!("\"version\": \"{new_version}\""),
        );

        if json.ends_with("package.json") {
            content = content
                .replace(
                    &format!("\"version\": \"{current_version}\""),
                    &format!("\"version\": \"{new_version}\""),
                )
                .replace(
                    &format!("\"version\": \"^{current_version}\""),
                    &format!("\"version\": \"^{new_version}\""),
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
        }

        std::fs::write(json, content)?;

        if json.ends_with("package.json") {
            std::process::Command::new("bash")
                .arg("-c")
                .arg("npm install")
                .current_dir(json.parent().expect("Parent directory not found"))
                .spawn()?
                .wait()?;
        }
    }
    Ok(())
}

fn update_api_md(schema: &std::path::PathBuf, api: &std::path::PathBuf) -> Result<(), CIError> {
    let schema_content = std::fs::read_to_string(schema)?;
    let api_content = std::fs::read_to_string(api)?;
    let api_docs = api_content
        .split_once("## openapi.json")
        .expect("invalid api.md")
        .0;
    let new_api = format!(
        "{}## openapi.json{LN}{LN}```json{LN}{}{LN}```",
        api_docs,
        schema_content.trim()
    );
    std::fs::write(api, new_api)?;
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
    update_tomls(&current_version, &new_version, &files.tomls)?;
    update_jsons(&current_version, &new_version, &files.jsons)?;
    update_api_md(&files.schema, &files.api)?;

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
