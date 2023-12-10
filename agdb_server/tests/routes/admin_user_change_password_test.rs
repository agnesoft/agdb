use crate::framework::TestServer;
use crate::framework::User;
use crate::framework::ADMIN_CHANGE_PASSWORD_URI;
use crate::framework::NO_TOKEN;
use crate::framework::USER_LOGIN_URI;

#[tokio::test]
async fn change_password() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let (name, _) = server.init_user().await?;
    let user = User {
        name: &name,
        password: "password456",
    };
    let admin = &server.admin_token;
    assert_eq!(
        server
            .post(ADMIN_CHANGE_PASSWORD_URI, &user, admin)
            .await?
            .0,
        201
    );
    assert_eq!(server.post(USER_LOGIN_URI, &user, NO_TOKEN).await?.0, 200);
    Ok(())
}

#[tokio::test]
async fn password_too_short() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let (name, _) = server.init_user().await?;
    let user = User {
        name: &name,
        password: "pswd",
    };
    let admin = &server.admin_token;
    assert_eq!(
        server
            .post(ADMIN_CHANGE_PASSWORD_URI, &user, admin)
            .await?
            .0,
        461
    );
    Ok(())
}

#[tokio::test]
async fn user_not_found() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = User {
        name: "unknown",
        password: "password456",
    };
    assert_eq!(
        server
            .post(ADMIN_CHANGE_PASSWORD_URI, &user, &server.admin_token)
            .await?
            .0,
        464
    );
    Ok(())
}

#[tokio::test]
async fn no_admin_token() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = User {
        name: "some_user",
        password: "password456",
    };
    assert_eq!(
        server
            .post(ADMIN_CHANGE_PASSWORD_URI, &user, NO_TOKEN)
            .await?
            .0,
        401
    );
    Ok(())
}
