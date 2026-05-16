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
    let owner = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.user_login(owner, owner).await?;
    let status = server.api.user_logout().await?;
    assert_eq!(status, 201);
    assert_eq!(server.api.token, None);
    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn no_token() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let status = server.api.user_logout().await.unwrap_err().status;
    assert_eq!(status, 401);

    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn logout_only_current_user_token() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let user = &next_user_name();

    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(user, user).await?;

    let mut client1 = crate::test_server::api_for_test(server.api.address());
    let mut client2 = crate::test_server::api_for_test(server.api.address());

    client1.user_login(user, user).await?;
    client2.user_login(user, user).await?;

    client1.user_status().await?;
    client2.user_status().await?;

    let status = client1.user_logout().await?;
    assert_eq!(status, 201);
    assert_eq!(client1.token, None);

    assert_eq!(client1.user_status().await.unwrap_err().status, 401);
    let status = client2.user_status().await?.1;
    assert_eq!(status.username, *user);
    assert!(status.login);
    assert!(!status.admin);

    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn logout_selected_session() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let user = &next_user_name();

    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(user, user).await?;

    let mut client1 = AgdbApi::new(
        ReqwestClient::with_user_agent(reqwest_client(), "session-owner"),
        server.api.address(),
    );
    let mut client2 = AgdbApi::new(
        ReqwestClient::with_user_agent(reqwest_client(), "session-revoked"),
        server.api.address(),
    );

    client1.user_login(user, user).await?;
    client2.user_login(user, user).await?;

    let status = client1.user_status().await?.1;
    let session_id = status
        .sessions
        .iter()
        .find(|s| s.agent == "session-revoked")
        .map(|s| s.id.clone())
        .expect("expected session for revoked client");

    let code = client1.user_logout_session(&session_id).await?;
    assert_eq!(code, 201);

    let status = client1.user_status().await?.1;
    assert_eq!(status.username, *user);
    assert!(status.login);
    assert!(!status.admin);

    assert_eq!(client2.user_status().await.unwrap_err().status, 401);

    Ok(())
}

#[cfg(feature = "api")]
pub fn test_defs() -> Vec<agdb::type_def::Type> {
    vec![
        __logout_type_def(),
        __no_token_type_def(),
        __logout_only_current_user_token_type_def(),
        __logout_selected_session_type_def(),
    ]
}
