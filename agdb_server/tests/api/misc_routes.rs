use crate::runner::Test;
use crate::runner::tests;
use crate::test_server::TestServer;
use crate::test_server::test_error::TestError;

async fn status() -> Result<(), TestError> {
    let server = TestServer::new().await?;
    let status = server.api.status().await?;
    assert_eq!(status, 200);
    Ok(())
}

async fn base_url() -> Result<(), TestError> {
    let server = TestServer::new().await?;
    assert_eq!(
        server.api.base_url(),
        format!("{}/api/v1", server.api.address())
    );
    Ok(())
}

async fn shutdown_no_token() -> Result<(), TestError> {
    let server = TestServer::new().await?;
    let status = server.api.admin_shutdown().await.unwrap_err().status;
    assert_eq!(status, 401);
    Ok(())
}

pub(super) const TESTS: &[Test] = tests![status, base_url, shutdown_no_token,];
