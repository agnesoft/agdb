use crate::runner::Test;
use crate::runner::tests;
use crate::test_server::ADMIN;
use crate::test_server::TestServer;
use crate::test_server::api_with_agent;
use crate::test_server::next_user_name;
use crate::test_server::test_error::TestError;

async fn user() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let user = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(user, user).await?;
    server.api.user_login(user, user).await?;
    let user_status = server.api.user_status().await?.1;
    assert_eq!(user_status.username, *user);
    assert!(user_status.login);
    assert!(!user_status.admin);
    assert_eq!(user_status.sessions.len(), 1);
    assert_eq!(
        user_status.sessions[0].agent,
        crate::test_server::test_agent_name()
    );
    Ok(())
}

async fn admin() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    server.api.user_login(ADMIN, ADMIN).await?;
    let user_status = server.api.user_status().await?.1;
    assert_eq!(user_status.username, ADMIN);
    assert!(user_status.login);
    assert!(user_status.admin);
    assert!(!user_status.sessions.is_empty());
    assert!(
        user_status
            .sessions
            .iter()
            .any(|session| session.agent == crate::test_server::test_agent_name())
    );

    Ok(())
}

async fn custom_agent() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let user = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(user, user).await?;

    let mut api = api_with_agent(server.api.address(), "custom-agent");
    api.user_login(user, user).await?;
    let user_status = api.user_status().await?.1;
    assert_eq!(user_status.sessions.len(), 1);
    assert_eq!(user_status.sessions[0].agent, "custom-agent");
    Ok(())
}

async fn no_token() -> Result<(), TestError> {
    let server = TestServer::new().await?;
    let status = server.api.user_status().await.unwrap_err().status;
    assert_eq!(status, 401);
    Ok(())
}

pub(super) const TESTS: &[Test] = tests![user, admin, custom_agent, no_token,];
