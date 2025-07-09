use std::io::Write;
use std::process::Command;

#[cfg(target_os = "windows")]
const BASH: &str = "C:/Program Files/Git/bin/bash.exe";
#[cfg(not(target_os = "windows"))]
const BASH: &str = "bash";

fn run_command(command: &str, dir: &str) {
    let out = Command::new(BASH)
        .arg("-c")
        .arg(command)
        .current_dir(dir)
        .output()
        .unwrap();
    std::io::stdout().write_all(&out.stdout).unwrap();
    std::io::stderr().write_all(&out.stderr).unwrap();
}

#[allow(dead_code)]
fn build_studio() {
    if std::env::var("AGDB_DOCKER_BUILD").is_err() {
        run_command(
            "pnpm i --frozen-lockfile && pnpm run build --filter=agdb_studio",
            "../",
        );
    }
}

fn main() {
    if !std::fs::exists("../agdb_studio/app/dist").unwrap() {
        #[cfg(feature = "studio")]
        build_studio();
    }
}
