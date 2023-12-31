use crate::TestServer;
use crate::DB_LIST_URI;
use crate::NO_TOKEN;

#[tokio::test]
async fn logout() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;

    let status = server
        .post::<()>(&format!("/user/{}/logout", &user.name), &None, &user.token)
        .await?
        .0;
    assert_eq!(status, 204);

    let status = server.get::<()>(DB_LIST_URI, &user.token).await?.0;
    assert_eq!(status, 401);

    Ok(())
}

#[tokio::test]
async fn user_not_found() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;

    let (status, _) = server
        .post::<()>("/user/not_found/login", &None, &user.token)
        .await?;
    assert_eq!(status, 404);

    Ok(())
}

#[tokio::test]
async fn no_token() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let status = server
        .post::<()>("/user/some_user/logout", &None, NO_TOKEN)
        .await?
        .0;
    assert_eq!(status, 401);

    Ok(())
}
