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
}

pub(crate) fn new() -> ServerResult<Config> {
    if let Ok(content) = std::fs::read_to_string(CONFIG_FILE) {
        return Ok(Config::new(serde_yaml::from_str(&content)?));
    }

    let config = ConfigImpl {
        host: "127.0.0.1".to_string(),
        port: 3000,
        admin: "admin".to_string(),
    };
    std::fs::write(CONFIG_FILE, serde_yaml::to_string(&config)?)?;

    Ok(Config::new(config))
}
