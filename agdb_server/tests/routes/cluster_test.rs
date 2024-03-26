use crate::TestServer;
use crate::TestServerImpl;
use crate::ADMIN;
use crate::HOST;
use crate::SERVER_DATA_DIR;
use agdb_api::AgdbApi;
use agdb_api::ReqwestClient;
use std::collections::HashMap;

#[tokio::test]
async fn db_cluster_established() -> anyhow::Result<()> {
    let port1 = TestServerImpl::next_port();
    let port2 = TestServerImpl::next_port();
    let port3 = TestServerImpl::next_port();
    let cluster = vec![
        format!("{HOST}:{port1}"),
        format!("{HOST}:{port2}"),
        format!("{HOST}:{port3}"),
    ];

    let mut config1 = HashMap::<&str, serde_yaml::Value>::new();
    config1.insert("host", HOST.into());
    config1.insert("port", port1.into());
    config1.insert("admin", ADMIN.into());
    config1.insert("data_dir", SERVER_DATA_DIR.into());
    config1.insert("cluster", cluster.into());

    let mut config2 = config1.clone();
    config2.insert("port", port2.into());

    let mut config3 = config1.clone();
    config3.insert("port", port3.into());

    let server1 = TestServerImpl::with_config(config1).await?;
    let server2 = TestServerImpl::with_config(config2).await?;
    let server3 = TestServerImpl::with_config(config3).await?;

    let client1 = AgdbApi::new(ReqwestClient::new(), &TestServer::url_base(), server1.port);
    let client2 = AgdbApi::new(ReqwestClient::new(), &TestServer::url_base(), server2.port);
    let client3 = AgdbApi::new(ReqwestClient::new(), &TestServer::url_base(), server3.port);

    let status1 = client1.status_cluster().await?;
    let status2 = client2.status_cluster().await?;
    let status3 = client3.status_cluster().await?;

    assert_eq!(status1.0, 200);
    assert_eq!(status2.0, 200);
    assert_eq!(status3.0, 200);

    assert_eq!(status1.1, status2.1);
    assert_eq!(status1.1, status3.1);

    assert!(status1.1.iter().any(|s| s.leader));

    Ok(())
}
