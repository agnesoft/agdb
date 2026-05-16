use crate::AgdbApi;
use crate::ReqwestClient;
use crate::test_server::ADMIN;
use crate::test_server::TestServer;
use crate::test_server::next_user_name;
use crate::test_server::reqwest_client;
use crate::test_server::test_error::TestError;

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn logout() -> Result<(), TestError> {
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

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn unknown_user() -> Result<(), TestError> {
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

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn non_admin() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let user = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(user, user).await?;
    server.api.user_login(user, user).await?;
    let status = server.api.admin_user_logout(user).await.unwrap_err().status;
    assert_eq!(status, 401);

    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn no_token() -> Result<(), TestError> {
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

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn logout_selected_session() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let user = &next_user_name();

    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(user, user).await?;

    let mut client1 = AgdbApi::new(
        ReqwestClient::with_user_agent(reqwest_client(), "admin-target-1"),
        server.api.address(),
    );
    let mut client2 = AgdbApi::new(
        ReqwestClient::with_user_agent(reqwest_client(), "admin-target-2"),
        server.api.address(),
    );

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
        .map(|s| s.id)
        .expect("expected session for revoked client");

    server
        .api
        .admin_user_logout_session(user, session_id)
        .await?;

    let status = client1.user_status().await?.1;
    assert_eq!(status.username, *user);
    assert!(status.login);

    assert_eq!(client2.user_status().await.unwrap_err().status, 401);

    Ok(())
}

#[cfg(feature = "api")]
pub fn test_defs() -> Vec<agdb::type_def::Type> {
    vec![
        __logout_type_def(),
        __unknown_user_type_def(),
        __non_admin_type_def(),
        __no_token_type_def(),
        __logout_selected_session_type_def(),
    ]
}
