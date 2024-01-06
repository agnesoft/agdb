use crate::ChangePassword;
use crate::TestServer;
use crate::UserLogin;
use crate::NO_TOKEN;

#[tokio::test]
async fn change_password() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let user = server.init_user().await?;
    let credentials = Some(UserLogin {
        username: &user.name,
        password: "password456",
    });
    let change: Option<ChangePassword> = Some(ChangePassword {
        password: &user.name,
        new_password: "password456",
    });
    assert_eq!(
        server
            .put("/user/change_password", &change, &user.token)
            .await?,
        201
    );
    assert_eq!(
        server.post("/user/login", &credentials, NO_TOKEN).await?.0,
        200
    );

    Ok(())
}

#[tokio::test]
async fn invalid_credentials() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let user = server.init_user().await?;
    let change = Some(ChangePassword {
        password: "bad_password",
        new_password: "password456",
    });
    assert_eq!(
        server
            .put("/user/change_password", &change, &user.token)
            .await?,
        401
    );

    Ok(())
}

#[tokio::test]
async fn password_too_short() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let user = server.init_user().await?;
    let change = Some(ChangePassword {
        password: &user.name,
        new_password: "pswd",
    });
    assert_eq!(
        server
            .put("/user/change_password", &change, &user.token)
            .await?,
        461
    );

    Ok(())
}

#[tokio::test]
async fn no_token() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let change = Some(ChangePassword {
        password: "password123",
        new_password: "password456",
    });
    assert_eq!(
        server
            .put("/user/change_password", &change, NO_TOKEN)
            .await?,
        401
    );

    Ok(())
}
