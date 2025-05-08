use std::sync::Arc;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use tracing::level_filters::LevelFilter;

pub(crate) type Config = Arc<ConfigImpl>;

pub(crate) const SALT_LEN: usize = 16;
pub(crate) const DEFAULT_LOG_BODY_LIMIT: u64 = 10 * 1024;
pub(crate) const DEFAULT_REQUEST_BODY_LIMIT: u64 = 10 * 1024 * 1024;

#[derive(Debug)]
pub struct ConfigImpl {
    pub(crate) bind: String,
    pub(crate) address: String,
    pub(crate) basepath: String,
    pub(crate) web_staticpaths: Vec<String>,
    pub(crate) admin: String,
    pub(crate) log_level: LevelFilter,
    pub(crate) log_body_limit: u64,
    pub(crate) request_body_limit: u64,
    pub(crate) data_dir: String,
    pub(crate) pepper_path: String,
    pub(crate) tls_certificate: String,
    pub(crate) tls_key: String,
    pub(crate) tls_root: String,
    pub(crate) cluster_token: String,
    pub(crate) cluster_heartbeat_timeout_ms: u64,
    pub(crate) cluster_term_timeout_ms: u64,
    pub(crate) cluster: Vec<String>,
    pub(crate) cluster_node_id: usize,
    pub(crate) start_time: u64,
    pub(crate) pepper: Option<[u8; SALT_LEN]>,
}

pub(crate) fn new(config_file: &str) -> Config {
    if let Ok(content) = std::fs::read_to_string(config_file) {
        let mut config_impl: ConfigImpl = from_str(&content);
        config_impl.cluster_node_id = config_impl
            .cluster
            .iter()
            .position(|x| x == &config_impl.address)
            .unwrap_or(0);
        config_impl.start_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        if !config_impl.pepper_path.is_empty() {
            let pepper_raw = std::fs::read(&config_impl.pepper_path).unwrap();
            let pepper = pepper_raw.trim_ascii();

            if pepper.len() != SALT_LEN {
                panic!(
                    "invalid pepper length {}, expected {SALT_LEN}",
                    pepper.len()
                );
            }

            config_impl.pepper = Some(
                pepper[0..SALT_LEN]
                    .try_into()
                    .expect("pepper length should be 16"),
            );
        }

        let config = Config::new(config_impl);

        if !config.cluster.is_empty() && !config.cluster.contains(&config.address) {
            panic!(
                "cluster does not contain local node: {} ({:?})",
                config.address, config.cluster
            );
        }

        return config;
    }

    let config = ConfigImpl {
        bind: ":::3000".to_string(),
        address: "http://localhost:3000".to_string(),
        basepath: "".to_string(),
        web_staticpaths: vec![],
        admin: "admin".to_string(),
        log_level: LevelFilter::INFO,
        log_body_limit: DEFAULT_LOG_BODY_LIMIT,
        request_body_limit: DEFAULT_REQUEST_BODY_LIMIT,
        data_dir: "agdb_server_data".to_string(),
        pepper_path: String::new(),
        tls_certificate: String::new(),
        tls_key: String::new(),
        tls_root: String::new(),
        cluster_token: "cluster".to_string(),
        cluster_heartbeat_timeout_ms: 1000,
        cluster_term_timeout_ms: 3000,
        cluster: vec![],
        cluster_node_id: 0,
        start_time: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        pepper: None,
    };

    std::fs::write(config_file, to_str(&config)).unwrap();

    Config::new(config)
}

pub(crate) fn cluster_from_value(value: &str) -> Vec<String> {
    let mut cluster = Vec::new();

    for node in value
        .trim_start_matches('[')
        .trim_end_matches(']')
        .trim()
        .split(',')
    {
        let node = node.trim().trim_matches(['\'', '"']);

        if node.is_empty() {
            continue;
        }

        cluster.push(node.to_string());
    }

    cluster
}

pub(crate) fn from_str(content: &str) -> ConfigImpl {
    let mut config = ConfigImpl {
        bind: String::new(),
        address: String::new(),
        basepath: String::new(),
        web_staticpaths: vec![],
        admin: String::new(),
        log_level: LevelFilter::INFO,
        log_body_limit: DEFAULT_LOG_BODY_LIMIT,
        request_body_limit: DEFAULT_REQUEST_BODY_LIMIT,
        data_dir: String::new(),
        pepper_path: String::new(),
        tls_certificate: String::new(),
        tls_key: String::new(),
        tls_root: String::new(),
        cluster_token: String::new(),
        cluster_heartbeat_timeout_ms: 0,
        cluster_term_timeout_ms: 0,
        cluster: vec![],
        cluster_node_id: 0,
        start_time: 0,
        pepper: None,
    };

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = line.split_once(':') {
            let key = key.trim().trim_matches(['\'', '"']);
            let value = if let Some((value, _comment)) = value.rsplit_once("#") {
                value
            } else {
                value
            };
            let value = value.trim().trim_matches(['\'', '"']).trim();

            match key {
                "bind" => config.bind = value.to_string(),
                "address" => config.address = value.to_string(),
                "basepath" => config.basepath = value.to_string(),
                "web_staticpaths" => {
                    config.web_staticpaths = cluster_from_value(value);
                },
                "admin" => config.admin = value.to_string(),
                "log_level" => config.log_level = level_filter_from_str(value),
                "log_body_limit" => config.log_body_limit = value.parse().unwrap(),
                "request_body_limit" => config.request_body_limit = value.parse().unwrap(),
                "data_dir" => config.data_dir = value.to_string(),
                "pepper_path" => config.pepper_path = value.to_string(),
                "tls_certificate" => config.tls_certificate = value.to_string(),
                "tls_key" => config.tls_key = value.to_string(),
                "tls_root" => config.tls_root = value.to_string(),
                "cluster_token" => config.cluster_token = value.to_string(),
                "cluster_heartbeat_timeout_ms" => {
                    config.cluster_heartbeat_timeout_ms = value.parse().unwrap()
                }
                "cluster_term_timeout_ms" => {
                    config.cluster_term_timeout_ms = value.parse().unwrap()
                }
                "cluster" => {
                    config.cluster = cluster_from_value(value);
                }
                _ => panic!("Unknown key: {}", key),
            }
        }
    }

    config
}

pub(crate) fn to_str(config: &ConfigImpl) -> String {
    let mut buffer = String::new();
    buffer.push_str(&format!("bind: {}\n", config.bind));
    buffer.push_str(&format!("address: {}\n", config.address));
    buffer.push_str(&format!("basepath: {}\n", config.basepath));
    buffer.push_str(&format!("web_staticpaths: [{}]\n", config.web_staticpaths.join(", ")));
    buffer.push_str(&format!("admin: {}\n", config.admin));
    buffer.push_str(&format!(
        "log_level: {}\n",
        level_filter_to_str(&config.log_level)
    ));
    buffer.push_str(&format!("log_body_limit: {}\n", config.log_body_limit));
    buffer.push_str(&format!(
        "request_body_limit: {}\n",
        config.request_body_limit
    ));
    buffer.push_str(&format!("data_dir: {}\n", config.data_dir));
    buffer.push_str(&format!("pepper_path: {}\n", config.pepper_path));
    buffer.push_str(&format!("tls_certificate: {}\n", config.tls_certificate));
    buffer.push_str(&format!("tls_key: {}\n", config.tls_key));
    buffer.push_str(&format!("tls_root: {}\n", config.tls_root));
    buffer.push_str(&format!("cluster_token: {}\n", config.cluster_token));

    buffer.push_str(&format!(
        "cluster_heartbeat_timeout_ms: {}\n",
        config.cluster_heartbeat_timeout_ms
    ));
    buffer.push_str(&format!(
        "cluster_term_timeout_ms: {}\n",
        config.cluster_term_timeout_ms
    ));
    buffer.push_str(&format!("cluster: [{}]\n", config.cluster.join(", ")));
    buffer
}

fn level_filter_to_str(level_filter: &LevelFilter) -> &str {
    match *level_filter {
        LevelFilter::OFF => "OFF",
        LevelFilter::ERROR => "ERROR",
        LevelFilter::WARN => "WARN",
        LevelFilter::INFO => "INFO",
        LevelFilter::DEBUG => "DEBUG",
        LevelFilter::TRACE => "TRACE",
    }
}

fn level_filter_from_str(level_filter: &str) -> LevelFilter {
    match level_filter {
        "OFF" => LevelFilter::OFF,
        "ERROR" => LevelFilter::ERROR,
        "WARN" => LevelFilter::WARN,
        "INFO" => LevelFilter::INFO,
        "DEBUG" => LevelFilter::DEBUG,
        "TRACE" => LevelFilter::TRACE,
        _ => panic!("Invalid log level: {}", level_filter),
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
        let _config = config::new(test_file.filename);
        assert!(std::fs::exists(test_file.filename).unwrap());
        let _config = config::new(test_file.filename);
    }

    #[test]
    #[should_panic]
    fn invalid_cluster() {
        let test_file = TestFile::new("test_config_invalid_cluster.yaml");
        let config = ConfigImpl {
            bind: ":::3000".to_string(),
            address: "http://localhost:3000".to_string(),
            basepath: "".to_string(),
            web_staticpaths: vec!["icetool".to_string()],
            admin: "admin".to_string(),
            log_level: LevelFilter::INFO,
            log_body_limit: DEFAULT_LOG_BODY_LIMIT,
            request_body_limit: DEFAULT_REQUEST_BODY_LIMIT,
            data_dir: "agdb_server_data".to_string(),
            pepper_path: String::new(),
            tls_certificate: String::new(),
            tls_key: String::new(),
            tls_root: String::new(),
            cluster_token: "cluster".to_string(),
            cluster_heartbeat_timeout_ms: 1000,
            cluster_term_timeout_ms: 3000,
            cluster: vec!["http://localhost:3001".to_string()],
            cluster_node_id: 0,
            start_time: 0,
            pepper: None,
        };
        std::fs::write(test_file.filename, to_str(&config)).unwrap();

        config::new(test_file.filename);
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
            web_staticpaths: vec![],
            admin: "admin".to_string(),
            log_level: LevelFilter::INFO,
            log_body_limit: DEFAULT_LOG_BODY_LIMIT,
            request_body_limit: DEFAULT_REQUEST_BODY_LIMIT,
            data_dir: "agdb_server_data".to_string(),
            pepper_path: pepper_file.filename.to_string(),
            tls_certificate: String::new(),
            tls_key: String::new(),
            tls_root: String::new(),
            cluster_token: "cluster".to_string(),
            cluster_heartbeat_timeout_ms: 1000,
            cluster_term_timeout_ms: 3000,
            cluster: vec![],
            cluster_node_id: 0,
            start_time: 0,
            pepper: None,
        };

        std::fs::write(test_file.filename, to_str(&config)).unwrap();

        let config = config::new(test_file.filename);

        assert_eq!(config.pepper.as_ref(), Some(pepper));
    }

    #[test]
    #[should_panic]
    fn pepper_missing() {
        let test_file = TestFile::new("pepper_missing.yaml");
        let config = ConfigImpl {
            bind: ":::3000".to_string(),
            address: "http://localhost:3000".to_string(),
            basepath: "".to_string(),
            web_staticpaths: vec![],
            admin: "admin".to_string(),
            log_level: LevelFilter::INFO,
            log_body_limit: DEFAULT_LOG_BODY_LIMIT,
            request_body_limit: DEFAULT_REQUEST_BODY_LIMIT,
            data_dir: "agdb_server_data".to_string(),
            pepper_path: "missing_file".to_string(),
            tls_certificate: String::new(),
            tls_key: String::new(),
            tls_root: String::new(),
            cluster_token: "cluster".to_string(),
            cluster_heartbeat_timeout_ms: 1000,
            cluster_term_timeout_ms: 3000,
            cluster: vec![],
            cluster_node_id: 0,
            start_time: 0,
            pepper: None,
        };
        std::fs::write(test_file.filename, to_str(&config)).unwrap();

        config::new(test_file.filename);
    }

    #[test]
    #[should_panic]
    fn pepper_invalid_len() {
        let test_file = TestFile::new("pepper_invalid_len.yaml");
        let pepper_file = TestFile::new("pepper_invalid_len");
        std::fs::write(pepper_file.filename, b"0123456789").unwrap();
        let config = ConfigImpl {
            bind: ":::3000".to_string(),
            address: "http://localhost:3000".to_string(),
            basepath: "".to_string(),
            web_staticpaths: vec![],
            admin: "admin".to_string(),
            log_level: LevelFilter::INFO,
            log_body_limit: DEFAULT_LOG_BODY_LIMIT,
            request_body_limit: DEFAULT_REQUEST_BODY_LIMIT,
            data_dir: "agdb_server_data".to_string(),
            pepper_path: pepper_file.filename.to_string(),
            tls_certificate: String::new(),
            tls_key: String::new(),
            tls_root: String::new(),
            cluster_token: "cluster".to_string(),
            cluster_heartbeat_timeout_ms: 1000,
            cluster_term_timeout_ms: 3000,
            cluster: vec![],
            cluster_node_id: 0,
            start_time: 0,
            pepper: None,
        };
        std::fs::write(test_file.filename, to_str(&config)).unwrap();

        config::new(test_file.filename);
    }
}
