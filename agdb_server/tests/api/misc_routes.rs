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

pub(super) const TESTS: &[Test] = tests![status,];
