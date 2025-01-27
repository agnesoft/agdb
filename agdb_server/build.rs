use std::process::Command;

fn build_studio() {
    #[cfg(feature = "studio")]
    {
        #[cfg(target_os = "windows")]
        const BASH: &str = "C:/Program Files/Git/bin/bash.exe";
        #[cfg(not(target_os = "windows"))]
        const BASH: &str = "bash";

        Command::new(BASH)
            .arg("-c")
            .arg("npm ci")
            .current_dir("../agdb_api/typescript")
            .output()
            .unwrap();

        Command::new(BASH)
            .arg("-c")
            .arg("npm build")
            .current_dir("../agdb_api/typescript")
            .output()
            .unwrap();

        Command::new(BASH)
            .arg("-c")
            .arg("npm ci")
            .current_dir("../agdb_studio")
            .output()
            .unwrap();

        Command::new(BASH)
            .arg("-c")
            .arg("npm build")
            .current_dir("../agdb_studio")
            .output()
            .unwrap();
    }
}

fn main() {
    println!("cargo::rerun-if-changed=../agdb_api/typescript");
    println!("cargo::rerun-if-changed=../agdb_studio");

    if std::env::var("AGDB_DOCKER_BUILD").is_err() {
        build_studio();
    }
}
