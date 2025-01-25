use crate::create_cluster;
use crate::root_ca;
use crate::wait_for_leader;
use crate::wait_for_ready;
use crate::TestServerImpl;
use crate::ADMIN;
use crate::SERVER_DATA_DIR;
use agdb_api::AgdbApi;
use agdb_api::ReqwestClient;
use std::collections::HashMap;
use std::time::Duration;

#[tokio::test]
async fn https() -> anyhow::Result<()> {
    let mut config = HashMap::<&str, serde_yml::Value>::new();
    let port = TestServerImpl::next_port();
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")?;

    config.insert("bind", format!(":::{port}").into());
    config.insert("address", format!("https://localhost:{port}").into());
    config.insert("data_dir", SERVER_DATA_DIR.into());
    config.insert("basepath", "".into());
    config.insert("admin", ADMIN.into());
    config.insert("log_level", "INFO".into());
    config.insert("pepper_path", "".into());
    config.insert(
        "tls_certificate",
        format!("{manifest_dir}/tests/test_cert.pem").into(),
    );
    config.insert(
        "tls_key",
        format!("{manifest_dir}/tests/test_cert.key.pem").into(),
    );
    config.insert(
        "tls_root",
        format!("{manifest_dir}/tests/test_root_ca.pem").into(),
    );
    config.insert("cluster_token", "test".into());
    config.insert("cluster_heartbeat_timeout_ms", 1000.into());
    config.insert("cluster_term_timeout_ms", 3000.into());
    config.insert("cluster", Vec::<String>::new().into());

    TestServerImpl::with_config(config).await?;

    Ok(())
}

#[tokio::test]
async fn cluster() -> anyhow::Result<()> {
    let mut servers = create_cluster(3, true).await?;
    let mut leader = AgdbApi::new(
        ReqwestClient::with_client(
            reqwest::Client::builder()
                .add_root_certificate(root_ca())
                .use_rustls_tls()
                .timeout(Duration::from_secs(30))
                .build()?,
        ),
        &servers[0].address,
    );
    leader.user_login(ADMIN, ADMIN).await?;
    leader.admin_shutdown().await?;
    servers[0].wait().await?;

    let mut statuses = Vec::with_capacity(servers.len() - 1);

    for server in &servers[1..] {
        let status = wait_for_leader(&AgdbApi::new(
            ReqwestClient::with_client(
                reqwest::Client::builder()
                    .add_root_certificate(root_ca())
                    .use_rustls_tls()
                    .timeout(Duration::from_secs(30))
                    .build()?,
            ),
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
            ReqwestClient::with_client(
                reqwest::Client::builder()
                    .add_root_certificate(root_ca())
                    .use_rustls_tls()
                    .timeout(Duration::from_secs(30))
                    .build()?,
            ),
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
