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
use tokio::net::TcpListener;
use tokio::sync::broadcast;

const CONFIG_FILE: &str = "agdb_server.yaml";

#[tokio::main]
async fn main() -> ServerResult {
    let config = config::new(CONFIG_FILE);
    tracing_subscriber::fmt()
        .with_max_level(config.log_level)
        .init();

    password::init(config.pepper);

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
    )?;
    let cluster_handle = cluster::start_with_shutdown(cluster, shutdown_receiver);

    tracing::info!("Process id: {}", std::process::id());
    tracing::info!(
        "Data directory: {}",
        std::env::current_dir()?.join(&config.data_dir).display()
    );

    #[cfg(feature = "tls")]
    if !config.tls_certificate.is_empty() && !config.tls_key.is_empty() {
        rustls::crypto::aws_lc_rs::default_provider()
            .install_default()
            .expect("Default Crypto Provider failed to install");

        let tls_config = axum_server::tls_rustls::RustlsConfig::from_pem_file(
            std::path::PathBuf::from(config.tls_certificate.clone()),
            std::path::PathBuf::from(config.tls_key.clone()),
        )
        .await?;
        let handle = axum_server::Handle::new();
        let shutdown_handle = handle.clone();

        tracing::info!("TLS enabled");

        tokio::spawn(async move {
            cluster_handle.await;
            shutdown_handle.graceful_shutdown(Some(std::time::Duration::from_secs(5)));
        });

        tracing::info!("Address: {}", config.address);
        tracing::info!("Listening at {}", config.bind);
        let listener = TcpListener::bind(&config.bind).await?.local_addr()?;
        return Ok(axum_server::bind_rustls(listener, tls_config)
            .handle(handle)
            .serve(app.into_make_service())
            .await?);
    }

    tracing::info!("Address: {}", config.address);
    tracing::info!("Listening at {}", config.bind);
    let listener = TcpListener::bind(&config.bind).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(cluster_handle)
        .await?;

    Ok(())
}
