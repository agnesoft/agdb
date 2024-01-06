use crate::TestServer;
use crate::UserCredentials;
use crate::UserLogin;
use crate::NO_TOKEN;

#[tokio::test]
async fn change_password() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let user = server.init_user().await?;
    let credentials = Some(UserCredentials {
        password: "password456",
    });

    assert_eq!(
        server
            .put(
                &format!("/admin/user/{}/change_password", user.name),
                &credentials,
                &server.admin_token
            )
            .await?,
        201
    );

    let login = Some(UserLogin {
        password: "password456",
        username: &user.name,
    });

    assert_eq!(server.post("/user/login", &login, NO_TOKEN).await?.0, 200);
    Ok(())
}

#[tokio::test]
async fn password_too_short() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let user = server.init_user().await?;
    let credentials = Some(UserCredentials { password: "pswd" });

    assert_eq!(
        server
            .put(
                &format!("/admin/user/{}/change_password", user.name),
                &credentials,
                &server.admin_token
            )
            .await?,
        461
    );
    Ok(())
}

#[tokio::test]
async fn user_not_found() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let credentials = Some(UserCredentials {
        password: "password456",
    });

    assert_eq!(
        server
            .put(
                "/admin/user/not_found/change_password",
                &credentials,
                &server.admin_token
            )
            .await?,
        404
    );
    Ok(())
}

#[tokio::test]
async fn non_admin() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let user = server.init_user().await?;
    let credentials = Some(UserCredentials {
        password: "password456",
    });
    assert_eq!(
        server
            .put(
                "/admin/user/not_found/change_password",
                &credentials,
                &user.token
            )
            .await?,
        401
    );
    Ok(())
}

#[tokio::test]
async fn no_token() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let credentials = Some(UserCredentials {
        password: "password456",
    });
    assert_eq!(
        server
            .put(
                "/admin/user/not_found/change_password",
                &credentials,
                NO_TOKEN
            )
            .await?,
        401
    );
    Ok(())
}
