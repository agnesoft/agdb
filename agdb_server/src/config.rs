use crate::server_error::ServerResult;
use serde::Deserialize;
use serde::Serialize;
use std::sync::Arc;

pub(crate) type Config = Arc<ConfigImpl>;

const CONFIG_FILE: &str = "agdb_server.yaml";

#[derive(Deserialize, Serialize)]
pub(crate) struct ConfigImpl {
    pub(crate) host: String,
    pub(crate) port: u16,
    pub(crate) admin: String,
    pub(crate) data_dir: String,
}

pub(crate) fn new() -> ServerResult<Config> {
    if let Ok(content) = std::fs::read_to_string(CONFIG_FILE) {
        return Ok(Config::new(serde_yaml::from_str(&content)?));
    }

    let config = ConfigImpl {
        host: "localhost".to_string(),
        port: 3000,
        admin: "admin".to_string(),
        data_dir: "agdb_server_data".to_string(),
    };

    std::fs::write(CONFIG_FILE, serde_yaml::to_string(&config)?)?;

    Ok(Config::new(config))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config;
    use std::path::Path;

    struct TestFile {}

    impl TestFile {
        fn new() -> Self {
            let _ = std::fs::remove_file(CONFIG_FILE);
            Self {}
        }
    }

    impl Drop for TestFile {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(CONFIG_FILE);
        }
    }

    #[test]
    fn default_values() {
        let _test_file = TestFile::new();
        assert!(!Path::new(CONFIG_FILE).exists());
        let _config = config::new().unwrap();
        assert!(Path::new(CONFIG_FILE).exists());
        let _config = config::new().unwrap();
    }
}
