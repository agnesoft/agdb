use crate::TestServer;
use crate::UserLogin;
use crate::NO_TOKEN;

#[tokio::test]
async fn login() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let credentials = Some(UserLogin {
        password: &user.name,
        username: &user.name,
    });
    let (status, token) = server.post("/user/login", &credentials, NO_TOKEN).await?;
    assert_eq!(status, 200);
    assert!(!token.is_empty());

    Ok(())
}

#[tokio::test]
async fn repeated_login() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let credentials = Some(UserLogin {
        password: &user.name,
        username: &user.name,
    });
    let (status, token) = server.post("/user/login", &credentials, NO_TOKEN).await?;
    assert_eq!(status, 200);
    assert!(!token.is_empty());

    let (status2, token2) = server.post("/user/login", &credentials, NO_TOKEN).await?;
    assert_eq!(status2, 200);
    assert_eq!(token, token2);

    Ok(())
}

#[tokio::test]
async fn invalid_credentials() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let credentials = Some(UserLogin {
        username: &user.name,
        password: "password456",
    });
    let (status, token) = server.post("/user/login", &credentials, NO_TOKEN).await?;
    assert_eq!(status, 401);
    assert_eq!(token, "unuauthorized");

    Ok(())
}

#[tokio::test]
async fn user_not_found() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = Some(UserLogin {
        password: "password456",
        username: "some_user",
    });
    let (status, _) = server.post("/user/login", &user, NO_TOKEN).await?;
    assert_eq!(status, 401);

    Ok(())
}
