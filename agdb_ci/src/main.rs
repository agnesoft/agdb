use std::io::BufRead;
use std::io::Write;
use std::path::Path;
use std::process::Command;

#[cfg(target_os = "windows")]
const BASH: &str = "C:/Program Files/Git/bin/bash.exe";
#[cfg(not(target_os = "windows"))]
const BASH: &str = "bash";

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

#[derive(Debug)]
struct CIError {
    #[allow(dead_code)]
    description: String,
}

impl<E: std::error::Error> From<E> for CIError {
    fn from(error: E) -> Self {
        Self {
            description: format!("{:?}", error),
        }
    }
}

fn current_version() -> Result<String, CIError> {
    Ok(std::process::Command::new("git")
        .arg("tag")
        .arg("--sort=taggerdate")
        .output()?
        .stdout
        .trim_ascii()
        .lines()
        .last()
        .ok_or(CIError {
            description: "tags not found".to_string(),
        })??[1..]
        .to_string())
}

fn new_version() -> Result<String, CIError> {
    Ok(std::fs::read_to_string(std::path::Path::new("Version"))?
        .trim()
        .to_string())
}

fn update_rust_project(
    toml: &Path,
    current_version: &str,
    new_version: &str,
) -> Result<(), CIError> {
    let mut content = std::fs::read_to_string(toml)?.replace(
        &format!("\nversion = \"{current_version}\""),
        &format!("\nversion = \"{new_version}\""),
    );
    for line in content.clone().lines() {
        let line = line.trim();
        if line.starts_with("agdb") {
            content = content.replace(line, &line.replace(current_version, new_version));
        }
    }
    std::fs::write(toml, content)?;

    Ok(())
}

fn update_npm_project(
    json: &Path,
    current_version: &str,
    new_version: &str,
) -> Result<(), CIError> {
    let content = std::fs::read_to_string(json)?.replace(
        &format!("\"version\": \"{current_version}\""),
        &format!("\"version\": \"{new_version}\""),
    );
    std::fs::write(json, content)?;

    let project_dir = json.parent().expect("Parent directory not found");
    println!(
        "Installing dependencies in '{}'",
        project_dir.to_string_lossy()
    );
    run_command(
        Command::new(BASH)
            .arg("-c")
            .arg("npm install")
            .current_dir(project_dir),
    )?;
    let _ = run_command(
        Command::new(BASH)
            .arg("-c")
            .arg("npm audit fix")
            .current_dir(project_dir),
    );

    Ok(())
}

fn update_projects(path: &Path, current_version: &str, new_version: &str) -> Result<(), CIError> {
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
            update_projects(path.as_path(), current_version, new_version)?;
        } else if filename.ends_with(".toml") {
            println!("Updating '{}'", path.to_string_lossy().replace('\\', "/"));
            update_rust_project(path.as_path(), current_version, new_version)?;
        } else if filename == "package.json" {
            println!("Updating '{}'", path.to_string_lossy().replace('\\', "/"));
            update_npm_project(path.as_path(), current_version, new_version)?;
        }
    }

    Ok(())
}

fn run_command(command: &mut Command) -> Result<(), CIError> {
    let out = command.output()?;
    std::io::stdout().write_all(&out.stdout)?;
    std::io::stderr().write_all(&out.stderr)?;
    if out.status.success() {
        Ok(())
    } else {
        Err(CIError {
            description: format!(
                "Command failed: {command:?} ({})",
                command
                    .get_current_dir()
                    .unwrap_or(Path::new("."))
                    .to_string_lossy()
            ),
        })
        .inspect(|e| {
            println!("{:?}", e);
        })
    }
}

fn main() -> Result<(), CIError> {
    println!(
        "PATH: {}",
        std::env::var("PATH").unwrap().replace(";", "\n")
    );

    let current_version = current_version()?;
    let new_version = new_version()?;
    println!("Current version: {}", current_version);
    println!("New version: {}", new_version);

    update_projects(Path::new("./"), &current_version, &new_version)?;

    println!("Generating openapi.json");
    run_command(
        Command::new("cargo")
            .arg("test")
            .arg("-p")
            .arg("agdb_server")
            .arg("api::tests::openapi")
            .arg("--")
            .arg("--exact"),
    )?;

    println!("Generating Typescript openapi");
    run_command(
        Command::new(BASH)
            .arg("-c")
            .arg("npm run openapi")
            .current_dir(Path::new("agdb_api").join("typescript")),
    )?;

    println!("Generating PHP openapi");
    run_command(
        Command::new(BASH)
            .arg("ci.sh")
            .arg("openapi")
            .current_dir(Path::new("agdb_api").join("php")),
    )?;

    println!("Generating test_queries.json");
    run_command(
        Command::new("cargo")
            .arg("test")
            .arg("-p")
            .arg("agdb_server")
            .arg("tests::test_queries")
            .arg("--")
            .arg("--exact"),
    )?;

    println!("Generating Typescript test_queries");
    run_command(
        Command::new(BASH)
            .arg("-c")
            .arg("npm run test_queries")
            .current_dir(Path::new("agdb_api").join("typescript")),
    )?;

    println!("Generating PHP test_queries");
    run_command(
        Command::new(BASH)
            .arg("ci.sh")
            .arg("test_queries")
            .current_dir(Path::new("agdb_api").join("php")),
    )?;

    println!("DONE");

    Ok(())
}
