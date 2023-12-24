use crate::TestServer;
use crate::NO_TOKEN;
use serde::Deserialize;
use serde::Serialize;
use std::path::Path;

#[derive(Serialize, Deserialize)]
struct RemoveDb {
    db: String,
}

#[tokio::test]
async fn remove() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let db = server.init_db("mapped", &user).await?;
    assert!(Path::new(&server.data_dir).join(&db).exists());
    assert_eq!(
        server
            .delete(&format!("/db/{db}/remove"), &user.token)
            .await?,
        204
    );
    assert!(Path::new(&server.data_dir).join(db).exists());
    Ok(())
}

#[tokio::test]
async fn db_not_found() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    assert_eq!(
        server
            .delete(&format!("/db/{}/not_found/remove", user.name), &user.token)
            .await?,
        404
    );
    Ok(())
}

#[tokio::test]
async fn non_owner() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let db = server.init_db("memory", &user).await?;
    let other = server.init_user().await?;
    assert_eq!(
        server
            .put::<()>(
                &format!("/db/{db}/user/{}/add?db_role=admin", other.name),
                &None,
                &user.token
            )
            .await?,
        201
    );
    assert_eq!(
        server
            .delete(&format!("/db/{db}/remove"), &other.token)
            .await?,
        403
    );
    Ok(())
}

#[tokio::test]
async fn no_token() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    assert_eq!(server.delete("/db/user/db/remove", NO_TOKEN).await?, 401);
    Ok(())
}
