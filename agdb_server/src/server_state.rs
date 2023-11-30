use crate::db::DbPool;
use axum::extract::FromRef;
use tokio::sync::broadcast::Sender;

#[derive(Clone)]
pub(crate) struct ServerState {
    pub(crate) db_pool: DbPool,
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
