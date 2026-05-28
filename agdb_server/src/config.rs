use agdb_api::LogLevelFilter;
use agdb_api::config_impl::ConfigImpl;
use agdb_api::config_impl::DEFAULT_LOG_BODY_LIMIT;
use agdb_api::config_impl::DEFAULT_REQUEST_BODY_LIMIT;
use agdb_api::config_impl::DEFAULT_TOKEN_EXPIRY_SECONDS;
use agdb_api::config_impl::MAX_TOKEN_EXPIRY_SECONDS;
use agdb_api::config_impl::MIN_TOKEN_EXPIRY_SECONDS;
use agdb_api::config_impl::SALT_LEN;
use std::sync::Arc;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

pub(crate) type Config = Arc<ConfigImpl>;

pub(crate) fn new(config_file: &str) -> Result<Config, String> {
    if let Ok(content) = std::fs::read_to_string(config_file) {
        let mut config_impl: ConfigImpl = from_str(&content)?;

        config_impl.cluster_node_id = config_impl
            .cluster
            .iter()
            .position(|x| x == &config_impl.server_url())
            .unwrap_or_default();

        config_impl.start_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| format!("Failed to get server start time since UNIX_EPOCH: {e:?}"))?
            .as_secs();

        if !config_impl.pepper_path.is_empty() {
            let pepper_raw = std::fs::read(&config_impl.pepper_path).map_err(|e| {
                format!(
                    "Failed to read the pepper file '{}': {e:?}",
                    config_impl.pepper_path
                )
            })?;
            let pepper = pepper_raw.trim_ascii();

            if pepper.len() != SALT_LEN {
                return Err(format!(
                    "invalid pepper length {}, expected {SALT_LEN}",
                    pepper.len()
                ));
            }

            config_impl.pepper = Some(
                pepper[0..SALT_LEN]
                    .try_into()
                    .expect("pepper length should be 16"),
            );
        }

        let config = Config::new(config_impl);

        if !config.cluster.is_empty() && !config.cluster.contains(&config.server_url()) {
            return Err(format!(
                "Cluster does not contain local node: {} ({:?})",
                config.server_url(),
                config.cluster
            ));
        }

        return Ok(config);
    }

    let mut config = default_config();
    config.start_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("Failed to get server start time since UNIX_EPOCH: {e:?}"))?
        .as_secs();

    std::fs::write(config_file, agdb_api::config_impl::config_to_str(&config))
        .map_err(|e| format!("Failed to write config file '{}': {e:?}", config_file))?;

    Ok(Config::new(config))
}

pub(crate) fn vec_from_str(value: &str) -> Vec<String> {
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

pub(crate) fn from_str(content: &str) -> Result<ConfigImpl, String> {
    let mut config = default_config();

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
                "basepath" => {
                    config.basepath = if value.is_empty() || value.starts_with('/') {
                        value.to_string()
                    } else {
                        format!("/{value}")
                    }
                    .trim_end_matches('/')
                    .to_string()
                }
                "static_roots" => config.static_roots = vec_from_str(value),
                "admin" => config.admin = value.to_string(),
                "log_level" => config.log_level = value.try_into()?,
                "log_body_limit" => {
                    config.log_body_limit = value
                        .parse()
                        .map_err(|e| format!("Invalid log_body_limit: {e:?}"))?
                }
                "request_body_limit" => {
                    config.request_body_limit = value
                        .parse()
                        .map_err(|e| format!("Invalid request_body_limit: {e:?}"))?
                }
                "data_dir" => config.data_dir = value.to_string(),
                "pepper_path" => config.pepper_path = value.to_string(),
                "tls_certificate" => config.tls_certificate = value.to_string(),
                "tls_key" => config.tls_key = value.to_string(),
                "tls_root" => config.tls_root = value.to_string(),
                "cluster_token" => config.cluster_token = value.to_string(),
                "cluster_heartbeat_timeout_ms" => {
                    config.cluster_heartbeat_timeout_ms = value
                        .parse()
                        .map_err(|e| format!("Invalid cluster_heartbeat_timeout_ms: {e:?}"))?
                }
                "cluster_term_timeout_ms" => {
                    config.cluster_term_timeout_ms = value
                        .parse()
                        .map_err(|e| format!("Invalid cluster_term_timeout_ms: {e:?}"))?
                }
                "cluster_election_factor_ms" => {
                    config.cluster_election_factor_ms = value
                        .parse()
                        .map_err(|e| format!("Invalid cluster_election_factor_ms: {e:?}"))?
                }
                "cluster" => config.cluster = vec_from_str(value),
                "token_expiry_seconds" => {
                    config.token_expiry_seconds = value
                        .parse()
                        .map_err(|e| format!("Invalid token_expiry_seconds: {e:?}"))?;
                    if config.token_expiry_seconds < MIN_TOKEN_EXPIRY_SECONDS
                        || config.token_expiry_seconds > MAX_TOKEN_EXPIRY_SECONDS
                    {
                        return Err(format!(
                            "token_expiry_seconds must be between {MIN_TOKEN_EXPIRY_SECONDS} and {MAX_TOKEN_EXPIRY_SECONDS}, got {}",
                            config.token_expiry_seconds
                        ));
                    }
                }
                _ => return Err(format!("Unknown key: {key}")),
            }
        }
    }

    normalize_address(&mut config);

    Ok(config)
}

fn normalize_address(config: &mut ConfigImpl) {
    if let Some((protocol, address)) = config.address.split_once("://") {
        if let Some((url, path)) = address.split_once('/') {
            if !path.is_empty() {
                crate::warn!(
                    "Path component in address is ignored, use 'basepath' to specify the path component of the address.",
                );
            }
            config.address = format!("{protocol}://{url}");
        }
    } else if let Some((address, _)) = config.address.split_once('/') {
        config.address = address.to_string();
    }
}

fn default_config() -> ConfigImpl {
    ConfigImpl {
        bind: ":::3000".to_string(),
        address: "http://localhost:3000".to_string(),
        basepath: "".to_string(),
        static_roots: Vec::new(),
        admin: "admin".to_string(),
        log_level: LogLevelFilter::Info,
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
        cluster_election_factor_ms: 1000,
        cluster: vec![],
        cluster_node_id: 0,
        start_time: 0,
        token_expiry_seconds: DEFAULT_TOKEN_EXPIRY_SECONDS,
        pepper: None,
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
        config::new(test_file.filename).unwrap();
        assert!(std::fs::exists(test_file.filename).unwrap());
        config::new(test_file.filename).unwrap();
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
            static_roots: vec![],
            admin: "admin".to_string(),
            log_level: LogLevelFilter::Info,
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
            cluster_election_factor_ms: 1000,
            cluster: vec![],
            cluster_node_id: 0,
            start_time: 0,
            token_expiry_seconds: DEFAULT_TOKEN_EXPIRY_SECONDS,
            pepper: None,
        };

        std::fs::write(
            test_file.filename,
            agdb_api::config_impl::config_to_str(&config),
        )
        .unwrap();

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
            static_roots: vec![],
            admin: "admin".to_string(),
            log_level: LogLevelFilter::Info,
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
            cluster_election_factor_ms: 1000,
            cluster: vec![],
            cluster_node_id: 0,
            start_time: 0,
            token_expiry_seconds: DEFAULT_TOKEN_EXPIRY_SECONDS,
            pepper: None,
        };
        std::fs::write(
            test_file.filename,
            agdb_api::config_impl::config_to_str(&config),
        )
        .unwrap();

        config::new(test_file.filename).unwrap_err();
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
            static_roots: vec![],
            admin: "admin".to_string(),
            log_level: LogLevelFilter::Info,
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
            cluster_election_factor_ms: 1000,
            cluster: vec![],
            cluster_node_id: 0,
            start_time: 0,
            token_expiry_seconds: DEFAULT_TOKEN_EXPIRY_SECONDS,
            pepper: None,
        };
        std::fs::write(
            test_file.filename,
            agdb_api::config_impl::config_to_str(&config),
        )
        .unwrap();

        config::new(test_file.filename).unwrap_err();
    }

    #[test]
    fn address_with_base_path_ignored() {
        let config_file = "address: http://localhost:3000/api";
        let config = config::from_str(config_file).unwrap();
        assert_eq!(config.address, "http://localhost:3000");
    }

    #[test]
    fn address_without_protocol_with_base_path_ignored() {
        let config_file = "address: localhost:3000/api";
        let config = config::from_str(config_file).unwrap();
        assert_eq!(config.address, "localhost:3000");
    }

    #[test]
    fn address_with_trailing_slash_ignored() {
        let config_file = "address: http://localhost:3000/";
        let config = config::from_str(config_file).unwrap();
        assert_eq!(config.address, "http://localhost:3000");
    }

    #[test]
    fn address_without_protocol_trailing_slash_ignored() {
        let config_file = "address: localhost:3000/";
        let config = config::from_str(config_file).unwrap();
        assert_eq!(config.address, "localhost:3000");
    }

    #[test]
    fn base_path_not_starting_with_slash_prepended() {
        let config_file = "basepath: api";
        let config = config::from_str(config_file).unwrap();
        assert_eq!(config.basepath, "/api");
    }

    #[test]
    fn base_path_ending_with_slash() {
        let config_file = "basepath: api/";
        let config = config::from_str(config_file).unwrap();
        assert_eq!(config.basepath, "/api");
    }

    #[test]
    fn server_url_with_base_path() {
        let config_file = "address: http://localhost:3000\nbasepath: /api/";
        let config = config::from_str(config_file).unwrap();
        assert_eq!(config.server_url(), "http://localhost:3000/api");
    }

    #[test]
    fn server_url_without_base_path() {
        let config_file = "address: http://localhost:3000";
        let config = config::from_str(config_file).unwrap();
        assert_eq!(config.server_url(), "http://localhost:3000");
    }

    #[test]
    fn cluster_node_id() {
        let test_file = TestFile::new("cluster_node_id.yaml");
        std::fs::write(test_file.filename, "address: http://localhost:3000\nbasepath: api/\ncluster: [http://localhost:3001, http://localhost:3000/api, http://localhost:3002]").unwrap();
        let config = config::new(test_file.filename).unwrap();
        assert_eq!(config.cluster_node_id, 1);
    }

    #[test]
    fn cluster_node_not_found() {
        let test_file = TestFile::new("cluster_node_not_found.yaml");
        std::fs::write(test_file.filename, "address: http://localhost:3000\nbasepath: api/\ncluster: [http://localhost:3001, http://localhost:3002]").unwrap();
        let err = config::new(test_file.filename).unwrap_err();
        assert_eq!(
            err,
            "Cluster does not contain local node: http://localhost:3000/api ([\"http://localhost:3001\", \"http://localhost:3002\"])"
        );
    }

    #[test]
    fn token_expiry_default() {
        let config = config::from_str("").unwrap();
        assert_eq!(config.token_expiry_seconds, DEFAULT_TOKEN_EXPIRY_SECONDS);
    }

    #[test]
    fn token_expiry_valid() {
        let config = config::from_str("token_expiry_seconds: 300").unwrap();
        assert_eq!(config.token_expiry_seconds, 300);
    }

    #[test]
    fn token_expiry_too_low() {
        let err = config::from_str("token_expiry_seconds: 30").unwrap_err();
        assert!(err.contains("token_expiry_seconds must be between"));
    }

    #[test]
    fn token_expiry_too_high() {
        let err = config::from_str("token_expiry_seconds: 100000").unwrap_err();
        assert!(err.contains("token_expiry_seconds must be between"));
    }
}
