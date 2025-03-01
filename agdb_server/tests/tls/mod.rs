use crate::create_cluster;
use crate::reqwest_client;
use crate::wait_for_leader;
use crate::wait_for_ready;
use crate::ConfigImpl;
use crate::TestServerImpl;
use crate::ADMIN;
use crate::DEFAULT_LOG_BODY_LIMIT;
use crate::DEFAULT_REQUEST_BODY_LIMIT;
use crate::SERVER_DATA_DIR;
use agdb_api::AgdbApi;
use agdb_api::ReqwestClient;
use tracing::level_filters::LevelFilter;

#[tokio::test]
async fn https() -> anyhow::Result<()> {
    let port = TestServerImpl::next_port();
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")?;
    let config = ConfigImpl {
        bind: format!(":::{port}"),
        address: format!("https://localhost:{port}"),
        basepath: String::new(),
        admin: ADMIN.to_string(),
        log_level: LevelFilter::INFO,
        log_body_limit: DEFAULT_LOG_BODY_LIMIT,
        request_body_limit: DEFAULT_REQUEST_BODY_LIMIT,
        data_dir: SERVER_DATA_DIR.into(),
        pepper_path: String::new(),
        tls_certificate: format!("{manifest_dir}/tests/test_cert.pem"),
        tls_key: format!("{manifest_dir}/tests/test_cert.key.pem"),
        tls_root: format!("{manifest_dir}/tests/test_root_ca.pem"),
        cluster_token: "test".to_string(),
        cluster_heartbeat_timeout_ms: 1000,
        cluster_term_timeout_ms: 3000,
        cluster: Vec::new(),
        cluster_node_id: 0,
        start_time: 0,
        pepper: None,
    };

    TestServerImpl::with_config(config).await?;

    Ok(())
}

#[tokio::test]
async fn cluster() -> anyhow::Result<()> {
    let mut servers = create_cluster(3, true).await?;
    let mut leader = AgdbApi::new(
        ReqwestClient::with_client(reqwest_client()),
        &servers[0].address,
    );
    leader.user_login(ADMIN, ADMIN).await?;
    leader.admin_shutdown().await?;
    servers[0].wait().await?;

    let mut statuses = Vec::with_capacity(servers.len() - 1);

    for server in &servers[1..] {
        let status = wait_for_leader(&AgdbApi::new(
            ReqwestClient::with_client(reqwest_client()),
            &server.address,
        ))
        .await?;
        statuses.push(status);
    }

    for status in &statuses {
        assert_eq!(statuses[0], *status);
    }

    servers[0].restart()?;
    wait_for_ready(&leader).await?;

    statuses.clear();

    for server in &servers {
        let status = wait_for_leader(&AgdbApi::new(
            ReqwestClient::with_client(reqwest_client()),
            &server.address,
        ))
        .await?;
        statuses.push(status);
    }

    for status in &statuses {
        assert_eq!(statuses[0], *status);
    }

    Ok(())
}
