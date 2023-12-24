use crate::TestServer;
use crate::UserCredentials;
use crate::NO_TOKEN;

#[tokio::test]
async fn add() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let credentials = Some(UserCredentials {
        password: "password123",
    });
    assert_eq!(
        server
            .post(
                "/admin/user/new_user/add",
                &credentials,
                &server.admin_token
            )
            .await?
            .0,
        201
    );
    assert_eq!(
        server
            .post(
                "/admin/user/new_user/add",
                &credentials,
                &server.admin_token
            )
            .await?
            .0,
        463
    );
    Ok(())
}

#[tokio::test]
async fn name_too_short() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let credentials = Some(UserCredentials {
        password: "password123",
    });
    assert_eq!(
        server
            .post("/admin/user/a/add", &credentials, &server.admin_token)
            .await?
            .0,
        462
    );
    Ok(())
}

#[tokio::test]
async fn password_too_short() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let credentials = Some(UserCredentials { password: "pswd" });
    assert_eq!(
        server
            .post("/admin/user/alice/add", &credentials, &server.admin_token)
            .await?
            .0,
        461
    );
    Ok(())
}

#[tokio::test]
async fn user_already_exists() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let credentials = Some(UserCredentials {
        password: "password123",
    });
    assert_eq!(
        server
            .post(
                &format!("/admin/user/{}/add", user.name),
                &credentials,
                &server.admin_token
            )
            .await?
            .0,
        463
    );
    Ok(())
}

#[tokio::test]
async fn no_admin_token() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let credentials = Some(UserCredentials {
        password: "password123",
    });
    assert_eq!(
        server
            .post("/admin/user/alice/add", &credentials, &user.token)
            .await?
            .0,
        401
    );
    Ok(())
}

#[tokio::test]
async fn no_token() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let credentials = Some(UserCredentials {
        password: "password123",
    });
    assert_eq!(
        server
            .post("/admin/user/alice/add", &credentials, NO_TOKEN)
            .await?
            .0,
        401
    );
    Ok(())
}
