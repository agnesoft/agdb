use crate::runner::Test;
use crate::runner::tests;
use crate::test_server::ADMIN;
use crate::test_server::TestServer;
use crate::test_server::next_user_name;
use crate::test_server::test_error::TestError;

async fn user_list() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let user1 = &next_user_name();
    let user2 = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(user1, user1).await?;
    server.api.admin_user_add(user2, user2).await?;
    let (status, list) = server.api.admin_user_list().await?;
    assert_eq!(status, 200);
    let admin_user = list
        .iter()
        .find(|u| u.username == ADMIN)
        .expect("the admin is listed");
    assert!(admin_user.login);
    assert!(admin_user.admin);
    assert!(!admin_user.sessions.is_empty());
    Ok(())
}

async fn non_admin() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let user = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(user, user).await?;
    server.api.user_login(user, user).await?;
    let status = server.api.admin_user_list().await.unwrap_err().status;
    assert_eq!(status, 401);
    Ok(())
}

async fn no_token() -> Result<(), TestError> {
    let server = TestServer::new().await?;
    let status = server.api.admin_user_list().await.unwrap_err().status;
    assert_eq!(status, 401);
    Ok(())
}

pub(super) const TESTS: &[Test] = tests![user_list, non_admin, no_token,];
