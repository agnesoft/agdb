use crate::TestServer;
use crate::DB_LIST_URI;
use crate::NO_TOKEN;

#[tokio::test]
async fn logout() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;

    let status = server.get::<()>(DB_LIST_URI, &user.token).await?.0;
    assert_eq!(status, 200);

    let status = server
        .post::<()>("/user/logout", &None, &user.token)
        .await?
        .0;
    assert_eq!(status, 201);

    let status = server.get::<()>(DB_LIST_URI, &user.token).await?.0;
    assert_eq!(status, 401);

    Ok(())
}

#[tokio::test]
async fn no_token() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let status = server.post::<()>("/user/logout", &None, NO_TOKEN).await?.0;
    assert_eq!(status, 401);

    Ok(())
}
