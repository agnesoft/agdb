use crate::runner::Test;
use crate::runner::tests;
use crate::test_server::ADMIN;
use crate::test_server::TestServer;
use crate::test_server::api_with_agent;
use crate::test_server::next_user_name;
use crate::test_server::test_error::TestError;

async fn logout() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let user = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(user, user).await?;
    server.api.user_login(user, user).await?;
    let token = server.api.token.clone();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_logout(user).await?;
    server.api.token = token;
    let status = server.api.db_list().await.unwrap_err().status;
    assert_eq!(status, 401);

    Ok(())
}

async fn unknown_user() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    server.api.user_login(ADMIN, ADMIN).await?;
    let status = server
        .api
        .admin_user_logout("unknown_user")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 404);

    Ok(())
}

async fn non_admin() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let user = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(user, user).await?;
    server.api.user_login(user, user).await?;
    let status = server.api.admin_user_logout(user).await.unwrap_err().status;
    assert_eq!(status, 401);

    Ok(())
}

async fn no_token() -> Result<(), TestError> {
    let server = TestServer::new().await?;
    let status = server
        .api
        .admin_user_logout("user")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 401);

    Ok(())
}

async fn logout_selected_session() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let user = &next_user_name();

    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(user, user).await?;

    let mut client1 = api_with_agent(server.api.address(), "admin-target-1");
    let mut client2 = api_with_agent(server.api.address(), "admin-target-2");

    client1.user_login(user, user).await?;
    client2.user_login(user, user).await?;

    let users = server.api.admin_user_list().await?.1;
    let target = users
        .into_iter()
        .find(|u| u.username == *user)
        .expect("expected target user in user list");
    let session_id = target
        .sessions
        .iter()
        .find(|s| s.agent == "admin-target-2")
        .map(|s| s.session.clone())
        .expect("expected session for revoked client");

    server
        .api
        .admin_user_logout_session(user, &session_id)
        .await?;

    let status = client1.user_status().await?.1;
    assert_eq!(status.username, *user);
    assert!(status.login);

    assert_eq!(client2.user_status().await.unwrap_err().status, 401);

    Ok(())
}

pub(super) const TESTS: &[Test] = tests![
    logout,
    unknown_user,
    non_admin,
    no_token,
    logout_selected_session,
];
