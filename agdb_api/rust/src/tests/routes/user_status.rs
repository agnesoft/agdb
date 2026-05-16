use crate::test_server::ADMIN;
use crate::test_server::TestServer;
use crate::test_server::next_user_name;
use crate::test_server::test_error::TestError;

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn user() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let user = &next_user_name();
    server.user_login(ADMIN).await?;
    server.api.admin_user_add(user, user).await?;
    server.user_login(user).await?;
    let user_status = server.api.user_status().await?.1;
    assert_eq!(user_status.username, *user);
    assert!(user_status.login);
    assert!(!user_status.admin);
    assert_eq!(user_status.sessions.len(), 1);
    assert_eq!(user_status.sessions[0].agent, "agdb_api");
    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn admin() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    server.user_login(ADMIN).await?;
    let user_status = server.api.user_status().await?.1;
    assert_eq!(user_status.username, ADMIN);
    assert!(user_status.login);
    assert!(user_status.admin);
    assert!(!user_status.sessions.is_empty());
    assert_eq!(user_status.sessions[0].agent, "agdb_api");

    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn custom_agent() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let user = &next_user_name();
    server.user_login(ADMIN).await?;
    server.api.admin_user_add(user, user).await?;

    let mut api = crate::AgdbApi::new(
        crate::ReqwestClient::with_user_agent(crate::test_server::reqwest_client(), "custom-agent"),
        server.api.address(),
    );
    api.user_login(user, user).await?;
    let user_status = api.user_status().await?.1;
    assert_eq!(user_status.sessions.len(), 1);
    assert_eq!(user_status.sessions[0].agent, "custom-agent");
    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn no_token() -> Result<(), TestError> {
    let server = TestServer::new().await?;
    let status = server.api.user_status().await.unwrap_err().status;
    assert_eq!(status, 401);
    Ok(())
}

#[cfg(feature = "api")]
pub fn test_defs() -> Vec<agdb::type_def::Type> {
    vec![
        __user_type_def(),
        __admin_type_def(),
        __no_token_type_def(),
        __custom_agent_type_def(),
    ]
}
