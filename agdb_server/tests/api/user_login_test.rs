use crate::runner::Test;
use crate::runner::tests;
use crate::test_server::ADMIN;
use crate::test_server::TestServer;
use crate::test_server::next_user_name;
use crate::test_server::test_error::TestError;

async fn login() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.user_login(owner, owner).await?;
    assert!(!server.api.token.clone().unwrap().is_empty());
    Ok(())
}

async fn repeated_login() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.user_login(owner, owner).await?;
    let token = server.api.token.clone().unwrap();
    server.api.user_login(owner, owner).await?;
    assert!(
        server.api.token.clone().unwrap() != token,
        "Login did not generate a new token: {token}"
    );
    Ok(())
}

async fn invalid_credentials() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    let status = server
        .api
        .user_login(owner, "bad_password")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 401);
    Ok(())
}

async fn user_not_found() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let status = server
        .api
        .user_login("owner", "bad_password")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 401);
    Ok(())
}

async fn concurrent_logins() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let user = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(user, user).await?;

    let mut handles = vec![];
    let mut apis = vec![];

    for _ in 0..3 {
        apis.push((
            crate::test_server::api_for_test(server.api.address()),
            user.to_string(),
        ));
    }

    for (mut api, user) in apis {
        handles.push(tokio::spawn(async move {
            api.user_login(&user, &user).await.unwrap();
            api.token.clone().unwrap_or_default()
        }));
    }

    let mut tokens = vec![];

    for handle in handles {
        tokens.push(handle.await?);
    }

    server.api.user_login(user, user).await?;
    let token = server.api.token.clone().unwrap();
    server.api.user_logout().await?;

    assert!(
        tokens.iter().all(|t| !t.is_empty()),
        "Some concurrent logins returned empty tokens: {:?}",
        tokens
    );

    let mut unique_tokens = tokens.clone();
    unique_tokens.sort();
    unique_tokens.dedup();
    let unique_count = unique_tokens.len();

    assert_eq!(
        unique_count,
        tokens.len(),
        "Concurrent logins produced duplicate tokens: {:?}",
        tokens
    );

    assert!(
        tokens.iter().all(|t| t != &token),
        "Follow-up login unexpectedly reused one of concurrent tokens: {:?}; new token: {}",
        tokens,
        token
    );

    Ok(())
}

pub(super) const TESTS: &[Test] = tests![
    login,
    repeated_login,
    invalid_credentials,
    user_not_found,
    concurrent_logins,
];
