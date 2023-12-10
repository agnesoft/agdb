use crate::framework::TestServer;
use crate::framework::User;
use crate::framework::NO_TOKEN;
use crate::framework::USER_LOGIN_URI;

#[tokio::test]
async fn login() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let (user, _) = server.init_user().await?;
    let user = User {
        name: &user,
        password: &user,
    };
    let (status, token) = server.post(USER_LOGIN_URI, &user, NO_TOKEN).await?;
    assert_eq!(status, 200);
    assert!(!token.is_empty());

    Ok(())
}

#[tokio::test]
async fn invalid_credentials() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let (user, _) = server.init_user().await?;
    let user = User {
        name: &user,
        password: "password456",
    };
    let (status, token) = server.post(USER_LOGIN_URI, &user, NO_TOKEN).await?;
    assert_eq!(status, 401);
    assert!(token.is_empty());

    Ok(())
}

#[tokio::test]
async fn user_not_found() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = User {
        name: "user_not_found",
        password: "password456",
    };
    let (status, _) = server.post(USER_LOGIN_URI, &user, NO_TOKEN).await?;
    assert_eq!(status, 464);

    Ok(())
}
