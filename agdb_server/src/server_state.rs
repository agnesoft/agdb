use crate::cluster::Cluster;
use crate::config::Config;
use crate::db_pool::DbPool;
use crate::server_db::ServerDb;
use axum::extract::FromRef;
use tokio::sync::broadcast::Sender;

#[derive(Clone)]
pub(crate) struct ServerState {
    pub(crate) db_pool: DbPool,
    pub(crate) config: Config,
    pub(crate) cluster: Cluster,
    pub(crate) server_db: ServerDb,
    pub(crate) shutdown_sender: Sender<()>,
}

impl FromRef<ServerState> for DbPool {
    fn from_ref(input: &ServerState) -> Self {
        input.db_pool.clone()
    }
}

impl FromRef<ServerState> for Cluster {
    fn from_ref(input: &ServerState) -> Self {
        input.cluster.clone()
    }
}

impl FromRef<ServerState> for Config {
    fn from_ref(input: &ServerState) -> Self {
        input.config.clone()
    }
}

impl FromRef<ServerState> for Sender<()> {
    fn from_ref(input: &ServerState) -> Self {
        input.shutdown_sender.clone()
    }
}

impl FromRef<ServerState> for ServerDb {
    fn from_ref(input: &ServerState) -> Self {
        input.server_db.clone()
    }
}
