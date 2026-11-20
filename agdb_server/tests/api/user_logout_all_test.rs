use crate::runner::Test;
use crate::runner::tests;
use crate::test_server::ADMIN;
use crate::test_server::TestServer;
use crate::test_server::api_with_agent;
use crate::test_server::next_user_name;
use crate::test_server::test_error::TestError;

async fn logout_all() -> Result<(), TestError> {
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

    let status = client1.user_logout_all().await?;
    assert_eq!(status, 201);
    assert_eq!(client1.token, None);

    assert_eq!(client1.user_status().await.unwrap_err().status, 401);
    assert_eq!(client2.user_status().await.unwrap_err().status, 401);

    Ok(())
}

async fn no_token() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let status = server.api.user_logout_all().await.unwrap_err().status;
    assert_eq!(status, 401);

    Ok(())
}

async fn logout_all_keep_self() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let user = &next_user_name();

    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(user, user).await?;

    let mut client1 = api_with_agent(server.api.address(), "logout-all-self");
    let mut client2 = api_with_agent(server.api.address(), "logout-all-other");

    client1.user_login(user, user).await?;
    client2.user_login(user, user).await?;

    let status = client1.user_logout_others().await?;
    assert_eq!(status, 201);
    assert!(client1.token.is_some());

    let status = client1.user_status().await?.1;
    assert_eq!(status.username, *user);
    assert!(status.login);

    assert_eq!(client2.user_status().await.unwrap_err().status, 401);

    Ok(())
}

pub(super) const TESTS: &[Test] = tests![logout_all, no_token, logout_all_keep_self,];
