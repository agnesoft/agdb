use agdb_api::AgdbApi;
use agdb_api::LogLevelFilter;
use agdb_api::ReqwestClient;
use agdb_api::config_impl::ConfigImpl;
use agdb_api::config_impl::DEFAULT_LOG_BODY_LIMIT;
use agdb_api::config_impl::DEFAULT_REQUEST_BODY_LIMIT;
use agdb_api::config_impl::DEFAULT_TOKEN_EXPIRY_SECONDS;
use agdb_api::test_server::ADMIN;
use agdb_api::test_server::SERVER_DATA_DIR;
use agdb_api::test_server::TestServerImpl;
use agdb_api::test_server::reqwest_client;
use agdb_api::test_server::test_cluster::create_cluster;
use agdb_api::test_server::test_cluster::wait_for_leader;
use agdb_api::test_server::wait_for_ready;

#[tokio::test]
async fn https() -> anyhow::Result<()> {
    let port = TestServerImpl::next_port();
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")?;
    let config = ConfigImpl {
        bind: format!(":::{port}"),
        address: format!("https://localhost:{port}"),
        basepath: String::new(),
        static_roots: Vec::new(),
        admin: ADMIN.to_string(),
        log_level: LogLevelFilter::Info,
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
        cluster_election_factor_ms: 1000,
        cluster: Vec::new(),
        cluster_node_id: 0,
        start_time: 0,
        token_expiry_seconds: DEFAULT_TOKEN_EXPIRY_SECONDS,
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
