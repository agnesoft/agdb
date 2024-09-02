use crate::TestServer;
use crate::TestServerImpl;
use crate::ADMIN;
use crate::HOST;
use crate::SERVER_DATA_DIR;
use agdb_api::AgdbApi;
use agdb_api::ReqwestClient;
use std::collections::HashMap;
use std::time::Duration;
use std::time::Instant;

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

    let now = Instant::now();
    let mut status1 = (0, vec![]);

    while now.elapsed().as_secs() < 3 {
        std::thread::sleep(Duration::from_millis(100));

        status1 = client1.cluster_status().await?;

        if status1.1.iter().any(|s| s.leader) {
            return Ok(());
        }
    }

    assert!(status1.1.iter().any(|s| s.leader));

    let client2 = AgdbApi::new(ReqwestClient::new(), &server2.address);
    let client3 = AgdbApi::new(ReqwestClient::new(), &server3.address);

    let status2 = client2.cluster_status().await?;
    let status3 = client3.cluster_status().await?;

    assert_eq!(status1, status2);
    assert_eq!(status1, status3);

    Ok(())
}

#[tokio::test]
async fn cluster_status() {
    let server = TestServer::new().await.unwrap();
    let (code, status) = server.api.cluster_status().await.unwrap();

    assert_eq!(code, 200);
    assert_eq!(status.len(), 0);
}

#[tokio::test]
async fn heartbeat_no_token() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let client = reqwest::Client::new();
    let status = client
        .post(server.full_url("/cluster/heartbeat?cluster_hash=test&term=1&leader=0"))
        .send()
        .await?
        .status();

    assert_eq!(status, 401);

    Ok(())
}

#[tokio::test]
async fn vote_no_token() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let client = reqwest::Client::new();
    let status = client
        .get(server.full_url("/cluster/vote?cluster_hash=test&term=1&leader=0"))
        .send()
        .await?
        .status();

    assert_eq!(status, 401);

    Ok(())
}
