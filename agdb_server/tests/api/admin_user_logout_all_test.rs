use crate::runner::Test;
use crate::runner::tests;
use crate::test_server::ADMIN;
use crate::test_server::TestServer;
use crate::test_server::next_user_name;
use crate::test_server::test_error::TestError;

async fn non_admin() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let user = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(user, user).await?;
    server.api.user_login(user, user).await?;
    let status = server.api.admin_user_logout_all().await.unwrap_err().status;
    assert_eq!(status, 401);

    Ok(())
}

async fn no_token() -> Result<(), TestError> {
    let server = TestServer::new().await?;
    let status = server.api.admin_user_logout_all().await.unwrap_err().status;
    assert_eq!(status, 401);

    Ok(())
}

pub(super) const TESTS: &[Test] = tests![non_admin, no_token,];
