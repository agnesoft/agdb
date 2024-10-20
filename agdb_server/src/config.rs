use crate::server_error::ServerError;
use crate::server_error::ServerResult;
use serde::Deserialize;
use serde::Serialize;
use std::sync::Arc;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use tracing::level_filters::LevelFilter;
use url::Url;

pub(crate) type Config = Arc<ConfigImpl>;

const CONFIG_FILE: &str = "agdb_server.yaml";

pub(crate) struct LogLevel(pub(crate) LevelFilter);

#[derive(Deserialize, Serialize)]
pub(crate) struct ConfigImpl {
    pub(crate) bind: String,
    pub(crate) address: Url,
    pub(crate) basepath: String,
    pub(crate) admin: String,
    pub(crate) log_level: LogLevel,
    pub(crate) data_dir: String,
    pub(crate) cluster_token: String,
    pub(crate) cluster: Vec<Url>,
    #[serde(skip)]
    pub(crate) cluster_node_id: usize,
    #[serde(skip)]
    pub(crate) start_time: u64,
}

pub(crate) fn new() -> ServerResult<Config> {
    if let Ok(content) = std::fs::read_to_string(CONFIG_FILE) {
        let mut config_impl: ConfigImpl = serde_yaml::from_str(&content)?;
        config_impl.cluster_node_id = config_impl
            .cluster
            .iter()
            .position(|x| x == &config_impl.address)
            .unwrap_or(0);
        config_impl.start_time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let config = Config::new(config_impl);

        if !config.cluster.is_empty() && !config.cluster.contains(&config.address) {
            return Err(ServerError::from(format!(
                "Cluster does not contain this node: {}",
                config.address
            )));
        }

        return Ok(config);
    }

    let config = ConfigImpl {
        bind: ":::3000".to_string(),
        address: Url::parse("localhost:3000")?,
        basepath: "".to_string(),
        admin: "admin".to_string(),
        log_level: LogLevel(LevelFilter::INFO),
        data_dir: "agdb_server_data".to_string(),
        cluster_token: "cluster".to_string(),
        cluster: vec![],
        cluster_node_id: 0,
        start_time: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
    };

    std::fs::write(CONFIG_FILE, serde_yaml::to_string(&config)?)?;

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

    #[test]
    fn log_level() {
        let level = LogLevel(LevelFilter::OFF);
        let serialized = serde_yaml::to_string(&level).unwrap();
        let other: LogLevel = serde_yaml::from_str(&serialized).unwrap();
        assert_eq!(level.0, other.0);

        let level = LogLevel(LevelFilter::ERROR);
        let serialized = serde_yaml::to_string(&level).unwrap();
        let other: LogLevel = serde_yaml::from_str(&serialized).unwrap();
        assert_eq!(level.0, other.0);

        let level = LogLevel(LevelFilter::WARN);
        let serialized = serde_yaml::to_string(&level).unwrap();
        let other: LogLevel = serde_yaml::from_str(&serialized).unwrap();
        assert_eq!(level.0, other.0);

        let level = LogLevel(LevelFilter::INFO);
        let serialized = serde_yaml::to_string(&level).unwrap();
        let other: LogLevel = serde_yaml::from_str(&serialized).unwrap();
        assert_eq!(level.0, other.0);

        let level = LogLevel(LevelFilter::DEBUG);
        let serialized = serde_yaml::to_string(&level).unwrap();
        let other: LogLevel = serde_yaml::from_str(&serialized).unwrap();
        assert_eq!(level.0, other.0);

        let level = LogLevel(LevelFilter::TRACE);
        let serialized = serde_yaml::to_string(&level).unwrap();
        let other: LogLevel = serde_yaml::from_str(&serialized).unwrap();
        assert_eq!(level.0, other.0);
    }
}
