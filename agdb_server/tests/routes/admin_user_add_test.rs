use crate::TestServer;
use crate::UserCredentials;
use crate::NO_TOKEN;

#[tokio::test]
async fn create_user() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let credentials = UserCredentials {
        password: "password123",
    };
    assert_eq!(
        server
            .put(
                "/admin/user/new_user/add",
                &credentials,
                &server.admin_token
            )
            .await?,
        201
    );
    Ok(())
}

#[tokio::test]
async fn name_too_short() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = UserCredentials {
        password: "password123",
    };
    assert_eq!(
        server
            .put("/admin/user/a/add", &user, &server.admin_token)
            .await?,
        462
    );
    Ok(())
}

#[tokio::test]
async fn password_too_short() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = UserCredentials { password: "pswd" };
    assert_eq!(
        server
            .put("/admin/user/alice/add", &user, &server.admin_token)
            .await?,
        461
    );
    Ok(())
}

#[tokio::test]
async fn user_already_exists() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let credentials = UserCredentials {
        password: "password123",
    };
    assert_eq!(
        server
            .put(
                &format!("/admin/user/{}/add", user.name),
                &credentials,
                &server.admin_token
            )
            .await?,
        463
    );
    Ok(())
}

#[tokio::test]
async fn no_admin_token() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = UserCredentials {
        password: "password123",
    };
    assert_eq!(
        server.put("/admin/user/alice/add", &user, NO_TOKEN).await?,
        401
    );
    Ok(())
}
