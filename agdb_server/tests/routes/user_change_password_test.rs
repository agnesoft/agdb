use crate::ChangePassword;
use crate::TestServer;
use crate::UserCredentials;
use crate::NO_TOKEN;

#[tokio::test]
async fn change_password() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let change = ChangePassword {
        password: &user.name,
        new_password: "password456",
    };
    let credentials = UserCredentials {
        password: "password456",
    };
    assert_eq!(
        server
            .post(
                &format!("/user/{}/change_password", user.name),
                &change,
                NO_TOKEN
            )
            .await?
            .0,
        201
    );
    assert_eq!(
        server
            .post(
                &format!("/user/{}/login", user.name),
                &credentials,
                NO_TOKEN
            )
            .await?
            .0,
        200
    );

    Ok(())
}

#[tokio::test]
async fn invalid_credentials() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let change = ChangePassword {
        password: "bad_password",
        new_password: "password456",
    };
    assert_eq!(
        server
            .post(
                &format!("/user/{}/change_password", user.name),
                &change,
                NO_TOKEN
            )
            .await?
            .0,
        401
    );

    Ok(())
}

#[tokio::test]
async fn password_too_short() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let change = ChangePassword {
        password: &user.name,
        new_password: "pswd",
    };
    assert_eq!(
        server
            .post(
                &format!("/user/{}/change_password", user.name),
                &change,
                NO_TOKEN
            )
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
        password: "password123",
        new_password: "password456",
    };
    assert_eq!(
        server
            .post("/user/not_found/change_password", &change, NO_TOKEN)
            .await?
            .0,
        464
    );

    Ok(())
}
