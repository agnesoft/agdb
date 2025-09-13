use crate::ADMIN;
use crate::TestServer;
use crate::next_user_name;

#[tokio::test]
async fn login() -> anyhow::Result<()> {
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

#[tokio::test]
async fn repeated_login() -> anyhow::Result<()> {
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

#[tokio::test]
async fn invalid_credentials() -> anyhow::Result<()> {
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

#[tokio::test]
async fn user_not_found() -> anyhow::Result<()> {
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

#[tokio::test]
async fn concurrent_logins() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let user = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(user, user).await?;

    let mut handles = vec![];
    let mut apis = vec![];

    for _ in 0..3 {
        apis.push((
            agdb_api::AgdbApi::new(
                agdb_api::ReqwestClient::with_client(crate::reqwest_client()),
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
