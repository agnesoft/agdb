pub mod framework;

use crate::framework::ChangePassword;
use crate::framework::TestServer;
use crate::framework::User;
use crate::framework::NO_TOKEN;
use crate::framework::USER_CHANGE_PASSWORD_URI;
use crate::framework::USER_LOGIN_URI;

#[tokio::test]
async fn change_password() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    server.init_user("alice", "password123").await?;
    let change = ChangePassword {
        name: "alice",
        password: "password123",
        new_password: "password456",
    };
    let user = User {
        name: "alice",
        password: "password456",
    };
    assert_eq!(
        server
            .post(USER_CHANGE_PASSWORD_URI, &change, NO_TOKEN)
            .await?
            .0,
        201
    );
    assert_eq!(server.post(USER_LOGIN_URI, &user, NO_TOKEN).await?.0, 200);

    Ok(())
}

#[tokio::test]
async fn invalid_credentials() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    server.init_user("alice", "password123").await?;
    let change = ChangePassword {
        name: "alice",
        password: "bad_password",
        new_password: "password456",
    };
    assert_eq!(
        server
            .post(USER_CHANGE_PASSWORD_URI, &change, NO_TOKEN)
            .await?
            .0,
        401
    );

    Ok(())
}

#[tokio::test]
async fn password_too_short() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    server.init_user("alice", "password123").await?;
    let change = ChangePassword {
        name: "alice",
        password: "password123",
        new_password: "pswd",
    };
    assert_eq!(
        server
            .post(USER_CHANGE_PASSWORD_URI, &change, NO_TOKEN)
            .await?
            .0,
        461
    );

    Ok(())
}

#[tokio::test]
async fn user_not_found() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let change = ChangePassword {
        name: "alice",
        password: "password123",
        new_password: "password456",
    };
    assert_eq!(
        server
            .post(USER_CHANGE_PASSWORD_URI, &change, NO_TOKEN)
            .await?
            .0,
        464
    );

    Ok(())
}
