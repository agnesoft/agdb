use crate::TestServer;
use crate::User;
use crate::NO_TOKEN;
use crate::USER_LOGIN_URI;

#[tokio::test]
async fn login() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let user = User {
        name: &user.name,
        password: &user.name,
    };
    let (status, token) = server.post(USER_LOGIN_URI, &user, NO_TOKEN).await?;
    assert_eq!(status, 200);
    assert!(!token.is_empty());

    Ok(())
}

#[tokio::test]
async fn invalid_credentials() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let user = User {
        name: &user.name,
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
