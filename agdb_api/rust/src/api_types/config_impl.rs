use crate::LogLevelFilter;

pub const SALT_LEN: usize = 16;
pub const DEFAULT_LOG_BODY_LIMIT: u64 = 10 * 1024;
pub const DEFAULT_REQUEST_BODY_LIMIT: u64 = 10 * 1024 * 1024;

#[derive(Debug)]
#[cfg_attr(feature = "api", derive(agdb::TypeDef))]
pub struct ConfigImpl {
    pub bind: String,
    pub address: String,
    pub basepath: String,
    pub static_roots: Vec<String>,
    pub admin: String,
    pub log_level: LogLevelFilter,
    pub log_body_limit: u64,
    pub request_body_limit: u64,
    pub data_dir: String,
    pub pepper_path: String,
    pub tls_certificate: String,
    pub tls_key: String,
    pub tls_root: String,
    pub cluster_token: String,
    pub cluster_heartbeat_timeout_ms: u64,
    pub cluster_term_timeout_ms: u64,
    pub cluster: Vec<String>,
    pub cluster_node_id: usize,
    pub start_time: u64,
    pub pepper: Option<[u8; SALT_LEN]>,
}

impl ConfigImpl {
    pub fn server_url(&self) -> String {
        format!("{}{}", self.address, self.basepath)
    }
}

#[cfg_attr(feature = "api", agdb::fn_def())]
pub fn config_to_str(config: &ConfigImpl) -> String {
    let mut buffer = String::new();
    buffer.push_str(&format!("bind: {}\n", config.bind));
    buffer.push_str(&format!("address: {}\n", config.address));
    buffer.push_str(&format!("basepath: {}\n", config.basepath));
    buffer.push_str(&format!(
        "static_roots: {}\n",
        config.static_roots.join(", ")
    ));
    buffer.push_str(&format!("admin: {}\n", config.admin));
    buffer.push_str(&format!("log_level: {}\n", config.log_level));
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
