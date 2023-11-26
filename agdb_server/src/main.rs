mod api;
mod app;
mod config;
mod db;
mod error;
mod logger;
mod password;
mod utilities;

use crate::db::DbPool;
use axum::Server;
use std::net::SocketAddr;
use tokio::signal;
use tokio::sync::broadcast::Receiver;
use tracing::Level;

const BIND_ADDRESS_ARRAY: [u8; 4] = [127, 0, 0, 1];
const BIND_ADDRESS: &str = "127.0.0.1";

async fn shutdown_signal(mut shutdown_shutdown: Receiver<()>) {
    tokio::select! {
        _ = signal::ctrl_c() => {},
        _ = shutdown_shutdown.recv() => {}
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    let (shutdown_sender, shutdown_receiver) = tokio::sync::broadcast::channel::<()>(1);
    let config = config::new()?;
    let db_pool = DbPool::new()?;
    let app = app::app(shutdown_sender, db_pool);
    tracing::info!("Listening at {BIND_ADDRESS}:{}", config.port);
    let addr = SocketAddr::from((BIND_ADDRESS_ARRAY, config.port));
    let shutdown = shutdown_signal(shutdown_receiver);

    Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown)
        .await?;

    Ok(())
}
