mod app;

use axum::Server;
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let (shutdown_sender, mut shutdown_receiver) = tokio::sync::broadcast::channel::<()>(1);
    let app = app::app(shutdown_sender);
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(async {
            let _ = shutdown_receiver.recv().await;
        })
        .await?;

    Ok(())
}
