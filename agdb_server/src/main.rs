mod action;
mod api;
mod app;
mod cluster;
mod config;
mod db_pool;
mod error_code;
mod forward;
mod logger;
mod password;
mod raft;
mod routes;
mod server_db;
mod server_error;
mod server_state;
mod user_id;
mod utilities;

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
    let db_pool = db_pool::new(config.clone(), &server_db).await?;
    let cluster = cluster::new(&config, &server_db, &db_pool).await?;

    let app = app::app(
        cluster.clone(),
        config.clone(),
        db_pool,
        server_db,
        shutdown_sender.clone(),
    );
    tracing::info!("Process id: {}", std::process::id());
    tracing::info!(
        "Data directory: {}",
        std::env::current_dir()?.join(&config.data_dir).display()
    );
    tracing::info!("Listening at {}", config.bind);
    let listener = tokio::net::TcpListener::bind(&config.bind).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(cluster::start_with_shutdown(cluster, shutdown_receiver))
        .await?;

    Ok(())
}
