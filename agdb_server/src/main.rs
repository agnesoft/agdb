mod api;
mod app;
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
use tokio::signal;
use tokio::sync::broadcast;
use tracing::Level;

async fn shutdown_signal(mut shutdown_receiver: broadcast::Receiver<()>) {
    tokio::select! {
        _ = signal::ctrl_c() => {},
        _ = shutdown_receiver.recv() => {},
    }
}

#[tokio::main]
async fn main() -> ServerResult {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    let (shutdown_sender, shutdown_receiver) = broadcast::channel::<()>(1);
    let config = config::new()?;
    let db_pool = DbPool::new(&config).await?;
    let address = format!("{}:{}", config.host, config.port);
    let app = app::app(config, shutdown_sender, db_pool);
    tracing::info!("Listening at {address}");
    let listener = tokio::net::TcpListener::bind(address).await?;

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal(shutdown_receiver))
        .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::api::Api;
    use std::fs::File;
    use std::io::Write;
    use utoipa::OpenApi;

    #[test]
    fn generate_openapi_schema() {
        let schema = Api::openapi().to_pretty_json().unwrap();
        let mut file = File::create("openapi/schema.json").unwrap();
        file.write_all(schema.as_bytes()).unwrap();
    }
}
