mod action;
mod api;
mod app;
mod cluster;
mod config;
mod db_pool;
mod error_code;
mod logger;
mod password;
mod raft;
mod redirect;
mod routes;
mod server_db;
mod server_error;
mod server_state;
mod user_id;
mod utilities;

use crate::db_pool::DbPool;
use server_error::ServerResult;
use tokio::sync::broadcast;

const CONFIG_FILE: &str = "agdb_server.yaml";

#[tokio::main]
async fn main() -> ServerResult {
    let config = config::new(CONFIG_FILE)?;
    tracing_subscriber::fmt()
        .with_max_level(config.log_level.0)
        .init();

    let (shutdown_sender, shutdown_receiver) = broadcast::channel::<()>(1);
    let server_db = server_db::new(&config).await?;
    let db_pool = DbPool::new(&config, &server_db).await?;
    let cluster = cluster::new(&config, &server_db, &db_pool).await?;

    let app = app::app(
        cluster.clone(),
        config.clone(),
        db_pool,
        server_db,
        shutdown_sender.clone(),
    );
    tracing::info!("Current directory: {}", std::env::current_dir()?.display());
    tracing::info!("Data directory: {}", config.data_dir);
    tracing::info!("Listening at {}", config.bind);
    let listener = tokio::net::TcpListener::bind(&config.bind).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(cluster::start_with_shutdown(cluster, shutdown_receiver))
        .await?;

    Ok(())
}
