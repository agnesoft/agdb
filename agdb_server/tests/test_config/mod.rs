use std::panic::Location;
use std::path::Path;

pub struct TestConfig {
    pub dir: String,
}

const CONFIG_FILE: &str = "agdb_server.yaml";

impl TestConfig {
    #[track_caller]
    pub fn new() -> Self {
        let caller = Location::caller();
        let dir = format!(
            "{}.{}.{}.test",
            Path::new(caller.file())
                .file_name()
                .unwrap()
                .to_str()
                .unwrap(),
            caller.line(),
            caller.column()
        );
        Self::remove_dir_if_exists(&dir);
        std::fs::create_dir(&dir).unwrap();

        Self { dir }
    }

    #[track_caller]
    pub fn new_content(content: &str) -> Self {
        let test_config = Self::new();

        std::fs::write(Path::new(&test_config.dir).join(CONFIG_FILE), content).unwrap();

        test_config
    }

    fn remove_dir_if_exists(dir: &str) {
        if Path::new(dir).exists() {
            std::fs::remove_dir_all(dir).unwrap();
        }
    }
}

impl Drop for TestConfig {
    fn drop(&mut self) {
        Self::remove_dir_if_exists(&self.dir);
    }
}
