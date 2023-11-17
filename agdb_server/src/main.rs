mod app;
mod logger;

use axum::Server;
use std::net::SocketAddr;
use tokio::signal;
use tokio::sync::broadcast::Receiver;
use tracing::Level;

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
    let app = app::app(shutdown_sender);
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let shutdown = shutdown_signal(shutdown_receiver);

    Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown)
        .await?;

    Ok(())
}
