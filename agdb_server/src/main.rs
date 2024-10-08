mod api;
mod app;
mod cluster;
mod config;
mod db_pool;
mod error_code;
mod logger;
mod password;
mod routes;
mod server_error;
mod server_state;
mod user_id;
mod utilities;

use crate::db_pool::DbPool;
use server_error::ServerResult;
use tokio::sync::broadcast;

#[tokio::main]
async fn main() -> ServerResult {
    let config = config::new()?;
    tracing_subscriber::fmt()
        .with_max_level(config.log_level.0)
        .init();

    let (shutdown_sender, shutdown_receiver) = broadcast::channel::<()>(1);
    let cluster = cluster::new(&config)?;
    let db_pool = DbPool::new(&config).await?;
    let app = app::app(
        config.clone(),
        shutdown_sender.clone(),
        db_pool,
        cluster.clone(),
    );
    tracing::info!("Listening at {}", config.bind);
    let listener = tokio::net::TcpListener::bind(&config.bind).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(cluster::start_with_shutdown(cluster, shutdown_receiver))
        .await?;

    Ok(())
}
