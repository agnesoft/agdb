use crate::password::SALT_LEN;
use crate::server_error::ServerError;
use crate::server_error::ServerResult;
use serde::Deserialize;
use serde::Serialize;
use std::sync::Arc;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use tracing::level_filters::LevelFilter;

pub(crate) type Config = Arc<ConfigImpl>;

#[derive(Debug)]
pub(crate) struct LogLevel(pub(crate) LevelFilter);

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct ConfigImpl {
    pub(crate) bind: String,
    pub(crate) address: String,
    pub(crate) basepath: String,
    pub(crate) admin: String,
    pub(crate) log_level: LogLevel,
    pub(crate) data_dir: String,
    pub(crate) pepper_path: String,
    pub(crate) cluster_token: String,
    pub(crate) cluster_heartbeat_timeout_ms: u64,
    pub(crate) cluster_term_timeout_ms: u64,
    pub(crate) cluster: Vec<String>,
    #[serde(skip)]
    pub(crate) cluster_node_id: usize,
    #[serde(skip)]
    pub(crate) start_time: u64,
    #[serde(skip)]
    pub(crate) pepper: Option<[u8; SALT_LEN]>,
}

pub(crate) fn new(config_file: &str) -> ServerResult<Config> {
    if let Ok(content) = std::fs::read_to_string(config_file) {
        let mut config_impl: ConfigImpl = serde_yml::from_str(&content)?;
        config_impl.cluster_node_id = config_impl
            .cluster
            .iter()
            .position(|x| x == &config_impl.address)
            .unwrap_or(0);
        config_impl.start_time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

        if !config_impl.pepper_path.is_empty() {
            let pepper_raw = std::fs::read(&config_impl.pepper_path)?;
            let pepper = pepper_raw.trim_ascii();

            if pepper.len() != SALT_LEN {
                return Err(ServerError::from(format!(
                    "invalid pepper length {}, expected 16",
                    pepper.len()
                )));
            }

            config_impl.pepper = Some(
                pepper[0..SALT_LEN]
                    .try_into()
                    .expect("pepper length should be 16"),
            );
        }

        let config = Config::new(config_impl);

        if !config.cluster.is_empty() && !config.cluster.contains(&config.address) {
            return Err(ServerError::from(format!(
                "cluster does not contain local node: {}",
                config.address
            )));
        }

        return Ok(config);
    }

    let config = ConfigImpl {
        bind: ":::3000".to_string(),
        address: "http://localhost:3000".to_string(),
        basepath: "".to_string(),
        admin: "admin".to_string(),
        log_level: LogLevel(LevelFilter::INFO),
        data_dir: "agdb_server_data".to_string(),
        pepper_path: String::new(),
        cluster_token: "cluster".to_string(),
        cluster_heartbeat_timeout_ms: 1000,
        cluster_term_timeout_ms: 3000,
        cluster: vec![],
        cluster_node_id: 0,
        start_time: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
        pepper: None,
    };

    std::fs::write(config_file, serde_yml::to_string(&config)?)?;

    Ok(Config::new(config))
}

impl Serialize for LogLevel {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self.0 {
            LevelFilter::OFF => serializer.serialize_str("OFF"),
            LevelFilter::ERROR => serializer.serialize_str("ERROR"),
            LevelFilter::WARN => serializer.serialize_str("WARN"),
            LevelFilter::INFO => serializer.serialize_str("INFO"),
            LevelFilter::DEBUG => serializer.serialize_str("DEBUG"),
            LevelFilter::TRACE => serializer.serialize_str("TRACE"),
        }
    }
}

impl<'de> Deserialize<'de> for LogLevel {
    fn deserialize<D>(deserializer: D) -> Result<LogLevel, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "OFF" => Ok(LogLevel(LevelFilter::OFF)),
            "ERROR" => Ok(LogLevel(LevelFilter::ERROR)),
            "WARN" => Ok(LogLevel(LevelFilter::WARN)),
            "INFO" => Ok(LogLevel(LevelFilter::INFO)),
            "DEBUG" => Ok(LogLevel(LevelFilter::DEBUG)),
            "TRACE" => Ok(LogLevel(LevelFilter::TRACE)),
            _ => Err(serde::de::Error::custom("Invalid log level")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config;

    struct TestFile {
        filename: &'static str,
    }

    impl TestFile {
        fn new(filename: &'static str) -> Self {
            let _ = std::fs::remove_file(filename);
            Self { filename }
        }
    }

    impl Drop for TestFile {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(self.filename);
        }
    }

    #[test]
    fn default_values() {
        let test_file = TestFile::new("test_config_default.yaml");
        assert!(!std::fs::exists(test_file.filename).unwrap());
        let _config = config::new(test_file.filename).unwrap();
        assert!(std::fs::exists(test_file.filename).unwrap());
        let _config = config::new(test_file.filename).unwrap();
    }

    #[test]
    fn invalid_cluster() {
        let test_file = TestFile::new("test_config_invalid_cluster.yaml");
        let config = ConfigImpl {
            bind: ":::3000".to_string(),
            address: "http://localhost:3000".to_string(),
            basepath: "".to_string(),
            admin: "admin".to_string(),
            log_level: LogLevel(LevelFilter::INFO),
            data_dir: "agdb_server_data".to_string(),
            pepper_path: String::new(),
            cluster_token: "cluster".to_string(),
            cluster_heartbeat_timeout_ms: 1000,
            cluster_term_timeout_ms: 3000,
            cluster: vec!["http://localhost:3001".to_string()],
            cluster_node_id: 0,
            start_time: 0,
            pepper: None,
        };
        std::fs::write(test_file.filename, serde_yml::to_string(&config).unwrap()).unwrap();
        assert_eq!(
            config::new(test_file.filename).unwrap_err().description,
            "cluster does not contain local node: http://localhost:3000"
        );
    }

    #[test]
    fn pepper_path() {
        let test_file = TestFile::new("pepper_path.yaml");
        let pepper_file = TestFile::new("pepper_path");
        let pepper = b"abcdefghijklmnop";
        std::fs::write(pepper_file.filename, pepper).unwrap();
        let config = ConfigImpl {
            bind: ":::3000".to_string(),
            address: "http://localhost:3000".to_string(),
            basepath: "".to_string(),
            admin: "admin".to_string(),
            log_level: LogLevel(LevelFilter::INFO),
            data_dir: "agdb_server_data".to_string(),
            pepper_path: pepper_file.filename.to_string(),
            cluster_token: "cluster".to_string(),
            cluster_heartbeat_timeout_ms: 1000,
            cluster_term_timeout_ms: 3000,
            cluster: vec![],
            cluster_node_id: 0,
            start_time: 0,
            pepper: None,
        };

        std::fs::write(test_file.filename, serde_yml::to_string(&config).unwrap()).unwrap();

        let config = config::new(test_file.filename).unwrap();

        assert_eq!(config.pepper.as_ref(), Some(pepper));
    }

    #[test]
    fn pepper_missing() {
        let test_file = TestFile::new("pepper_missing.yaml");
        let config = ConfigImpl {
            bind: ":::3000".to_string(),
            address: "http://localhost:3000".to_string(),
            basepath: "".to_string(),
            admin: "admin".to_string(),
            log_level: LogLevel(LevelFilter::INFO),
            data_dir: "agdb_server_data".to_string(),
            pepper_path: "missing_file".to_string(),
            cluster_token: "cluster".to_string(),
            cluster_heartbeat_timeout_ms: 1000,
            cluster_term_timeout_ms: 3000,
            cluster: vec![],
            cluster_node_id: 0,
            start_time: 0,
            pepper: None,
        };
        std::fs::write(test_file.filename, serde_yml::to_string(&config).unwrap()).unwrap();

        assert!(config::new(test_file.filename).is_err());
    }

    #[test]
    fn pepper_invalid_len() {
        let test_file = TestFile::new("pepper_invalid_len.yaml");
        let pepper_file = TestFile::new("pepper_invalid_len");
        std::fs::write(pepper_file.filename, b"0123456789").unwrap();
        let config = ConfigImpl {
            bind: ":::3000".to_string(),
            address: "http://localhost:3000".to_string(),
            basepath: "".to_string(),
            admin: "admin".to_string(),
            log_level: LogLevel(LevelFilter::INFO),
            data_dir: "agdb_server_data".to_string(),
            pepper_path: pepper_file.filename.to_string(),
            cluster_token: "cluster".to_string(),
            cluster_heartbeat_timeout_ms: 1000,
            cluster_term_timeout_ms: 3000,
            cluster: vec![],
            cluster_node_id: 0,
            start_time: 0,
            pepper: None,
        };
        std::fs::write(test_file.filename, serde_yml::to_string(&config).unwrap()).unwrap();

        assert_eq!(
            config::new(test_file.filename).unwrap_err().description,
            "invalid pepper length 10, expected 16"
        );
    }

    #[test]
    fn log_level() {
        let level = LogLevel(LevelFilter::OFF);
        let serialized = serde_yml::to_string(&level).unwrap();
        let other: LogLevel = serde_yml::from_str(&serialized).unwrap();
        assert_eq!(level.0, other.0);

        let level = LogLevel(LevelFilter::ERROR);
        let serialized = serde_yml::to_string(&level).unwrap();
        let other: LogLevel = serde_yml::from_str(&serialized).unwrap();
        assert_eq!(level.0, other.0);

        let level = LogLevel(LevelFilter::WARN);
        let serialized = serde_yml::to_string(&level).unwrap();
        let other: LogLevel = serde_yml::from_str(&serialized).unwrap();
        assert_eq!(level.0, other.0);

        let level = LogLevel(LevelFilter::INFO);
        let serialized = serde_yml::to_string(&level).unwrap();
        let other: LogLevel = serde_yml::from_str(&serialized).unwrap();
        assert_eq!(level.0, other.0);

        let level = LogLevel(LevelFilter::DEBUG);
        let serialized = serde_yml::to_string(&level).unwrap();
        let other: LogLevel = serde_yml::from_str(&serialized).unwrap();
        assert_eq!(level.0, other.0);

        let level = LogLevel(LevelFilter::TRACE);
        let serialized = serde_yml::to_string(&level).unwrap();
        let other: LogLevel = serde_yml::from_str(&serialized).unwrap();
        assert_eq!(level.0, other.0);

        let serialized = "INVALID".to_string();
        assert_eq!(
            serde_yml::from_str::<LogLevel>(&serialized)
                .unwrap_err()
                .to_string(),
            "Invalid log level"
        );
    }
}
