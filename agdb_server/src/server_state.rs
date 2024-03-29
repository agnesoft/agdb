use crate::config::Config;
use crate::db_pool::DbPool;
use axum::extract::FromRef;
use tokio::sync::broadcast::Sender;

#[derive(Clone)]
pub(crate) struct ServerState {
    pub(crate) db_pool: DbPool,
    pub(crate) config: Config,
    pub(crate) shutdown_sender: Sender<()>,
}

impl FromRef<ServerState> for DbPool {
    fn from_ref(input: &ServerState) -> Self {
        input.db_pool.clone()
    }
}

impl FromRef<ServerState> for Sender<()> {
    fn from_ref(input: &ServerState) -> Self {
        input.shutdown_sender.clone()
    }
}

impl FromRef<ServerState> for Config {
    fn from_ref(input: &ServerState) -> Self {
        input.config.clone()
    }
}
