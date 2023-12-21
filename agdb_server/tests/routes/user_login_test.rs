use crate::TestServer;
use crate::UserCredentials;
use crate::NO_TOKEN;

#[tokio::test]
async fn login() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let credentials = UserCredentials {
        password: &user.name,
    };
    let (status, token) = server
        .post(
            &format!("/user/{}/login", user.name),
            &credentials,
            NO_TOKEN,
        )
        .await?;
    assert_eq!(status, 200);
    assert!(!token.is_empty());

    Ok(())
}

#[tokio::test]
async fn invalid_credentials() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let credentials = UserCredentials {
        password: "password456",
    };
    let (status, token) = server
        .post(
            &format!("/user/{}/login", user.name),
            &credentials,
            NO_TOKEN,
        )
        .await?;
    assert_eq!(status, 401);
    assert!(token.is_empty());

    Ok(())
}

#[tokio::test]
async fn user_not_found() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = UserCredentials {
        password: "password456",
    };
    let (status, _) = server
        .post("/user/not_found/login", &user, NO_TOKEN)
        .await?;
    assert_eq!(status, 464);

    Ok(())
}
