use crate::TestServerImpl;
use crate::ADMIN;
use crate::SERVER_DATA_DIR;
use std::collections::HashMap;
use std::env;

#[tokio::test]
async fn https() -> anyhow::Result<()> {
    let mut config = HashMap::<&str, serde_yml::Value>::new();
    let port = TestServerImpl::next_port();
    let manifest_dir = env::var("CARGO_MANIFEST_DIR")?;

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
        format!("{manifest_dir}/tests/test_key.pem").into(),
    );
    config.insert("cluster_token", "test".into());
    config.insert("cluster_heartbeat_timeout_ms", 1000.into());
    config.insert("cluster_term_timeout_ms", 3000.into());
    config.insert("cluster", Vec::<String>::new().into());

    TestServerImpl::with_config(config).await?;

    Ok(())
}
