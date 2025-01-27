use std::io::Write;
use std::path::Path;
use std::process::Command;

#[allow(dead_code)]
fn run_command(command: &mut Command) {
    let out = command.output().unwrap();
    std::io::stdout().write_all(&out.stdout).unwrap();
    std::io::stderr().write_all(&out.stderr).unwrap();
    if !out.status.success() {
        println!(
            "Command failed: {command:?} ({})",
            command
                .get_current_dir()
                .unwrap_or(Path::new("."))
                .to_string_lossy()
        );
    }
}

#[allow(dead_code)]
fn build_studio() {
    println!("cargo::rerun-if-changed=../agdb_api/typescript");
    println!("cargo::rerun-if-changed=../agdb_studio");

    if std::env::var("AGDB_DOCKER_BUILD").is_err() {
        #[cfg(target_os = "windows")]
        const BASH: &str = "C:/Program Files/Git/bin/bash.exe";
        #[cfg(not(target_os = "windows"))]
        const BASH: &str = "bash";

        run_command(
            Command::new(BASH)
                .arg("-c")
                .arg("npm ci")
                .current_dir("../agdb_api/typescript"),
        );

        run_command(
            Command::new(BASH)
                .arg("-c")
                .arg("npm run build")
                .current_dir("../agdb_api/typescript"),
        );

        run_command(
            Command::new(BASH)
                .arg("-c")
                .arg("npm ci")
                .current_dir("../agdb_studio"),
        );

        run_command(
            Command::new(BASH)
                .arg("-c")
                .arg("npm run build")
                .current_dir("../agdb_studio"),
        );
    }
}

fn main() {
    #[cfg(feature = "studio")]
    build_studio();
}
