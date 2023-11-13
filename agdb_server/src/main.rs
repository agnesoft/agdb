mod app;

use axum::Server;
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let app = app::app();
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    Server::bind(&addr).serve(app.into_make_service()).await?;

    Ok(())
}
