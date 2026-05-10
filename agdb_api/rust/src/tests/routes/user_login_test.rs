use crate::test_server::ADMIN;
use crate::test_server::TestServer;
use crate::test_server::next_user_name;
use crate::test_server::test_error::TestError;

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn login() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let status = server.api.user_login(ADMIN, ADMIN).await?;
    assert_eq!(status, 200);
    server.api.admin_user_add(owner, owner).await?;
    let status = server.api.user_login(owner, owner).await?;
    assert_eq!(status, 200);
    assert!(!server.api.token.clone().unwrap().is_empty());
    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn repeated_login() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let status = server.api.user_login(ADMIN, ADMIN).await?;
    assert_eq!(status, 200);
    server.api.admin_user_add(owner, owner).await?;
    let status = server.api.user_login(owner, owner).await?;
    assert_eq!(status, 200);
    let token = server.api.token.clone().unwrap();
    let status = server.api.user_login(owner, owner).await?;
    assert_eq!(status, 200);
    assert_eq!(server.api.token.clone().unwrap(), token);
    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn invalid_credentials() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let status = server.api.user_login(ADMIN, ADMIN).await?;
    assert_eq!(status, 200);
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

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn user_not_found() -> Result<(), TestError> {
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

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn concurrent_logins() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let user = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(user, user).await?;

    let mut handles = vec![];
    let mut apis = vec![];

    for _ in 0..3 {
        apis.push((
            crate::AgdbApi::new(
                crate::ReqwestClient::with_client(crate::test_server::reqwest_client()),
                server.api.address(),
            ),
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
        tokens.iter().all(|t| t == &token),
        "Not all tokens are the same: {:?}",
        tokens
    );

    Ok(())
}

#[cfg(feature = "api")]
pub fn test_defs() -> Vec<agdb::type_def::Type> {
    vec![
        __login_type_def(),
        __repeated_login_type_def(),
        __invalid_credentials_type_def(),
        __user_not_found_type_def(),
        __concurrent_logins_type_def(),
    ]
}
