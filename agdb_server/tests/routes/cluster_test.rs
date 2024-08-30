use crate::TestServer;
use crate::TestServerImpl;
use crate::ADMIN;
use crate::HOST;
use crate::SERVER_DATA_DIR;
use agdb_api::AgdbApi;
use agdb_api::ReqwestClient;
use std::collections::HashMap;

#[tokio::test]
async fn cluster_established() -> anyhow::Result<()> {
    let port1 = TestServerImpl::next_port();
    let port2 = TestServerImpl::next_port();
    let port3 = TestServerImpl::next_port();
    let cluster = vec![
        format!("http://{HOST}:{port1}"),
        format!("http://{HOST}:{port2}"),
        format!("http://{HOST}:{port3}"),
    ];

    let mut config1 = HashMap::<&str, serde_yaml::Value>::new();
    config1.insert("bind", format!("{HOST}:{port1}").into());
    config1.insert("address", format!("http://{HOST}:{port1}").into());
    config1.insert("admin", ADMIN.into());
    config1.insert("basepath", "".into());
    config1.insert("data_dir", SERVER_DATA_DIR.into());
    config1.insert("cluster_token", "test".into());
    config1.insert("cluster", cluster.into());

    let mut config2 = config1.clone();
    config2.insert("bind", format!("{HOST}:{port2}").into());
    config2.insert("address", format!("http://{HOST}:{port2}").into());

    let mut config3 = config1.clone();
    config3.insert("bind", format!("{HOST}:{port3}").into());
    config3.insert("address", format!("http://{HOST}:{port3}").into());

    let server1 = TestServerImpl::with_config(config1).await?;
    let server2 = TestServerImpl::with_config(config2).await?;
    let server3 = TestServerImpl::with_config(config3).await?;

    let client1 = AgdbApi::new(ReqwestClient::new(), &server1.address);
    let client2 = AgdbApi::new(ReqwestClient::new(), &server2.address);
    let client3 = AgdbApi::new(ReqwestClient::new(), &server3.address);

    let status1 = client1.cluster_status().await?;
    let status2 = client2.cluster_status().await?;
    let status3 = client3.cluster_status().await?;

    assert_eq!(status1.0, 200);
    assert_eq!(status2.0, 200);
    assert_eq!(status3.0, 200);

    assert_eq!(status1.1, status2.1);
    assert_eq!(status1.1, status3.1);

    assert!(status1.1.iter().any(|s| s.leader));

    Ok(())
}

#[tokio::test]
async fn cluster_user() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let client = reqwest::Client::new();
    let status = client
        .get(server.full_url("/cluster/heartbeat"))
        .bearer_auth("test")
        .send()
        .await?
        .status();

    assert_eq!(status, 200);

    Ok(())
}

#[tokio::test]
async fn cluster_user_no_token() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let client = reqwest::Client::new();
    let status = client
        .get(server.full_url("/cluster/heartbeat"))
        .send()
        .await?
        .status();

    assert_eq!(status, 401);

    Ok(())
}
