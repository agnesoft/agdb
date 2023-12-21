use crate::DbWithRole;
use crate::TestServer;
use crate::DB_LIST_URI;
use crate::NO_TOKEN;
use serde::Deserialize;
use serde::Serialize;
use std::path::Path;

#[derive(Serialize, Deserialize)]
struct DeleteDb {
    db: String,
}

#[tokio::test]
async fn delete() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let db = server.init_db("mapped", &user).await?;
    assert!(Path::new(&server.data_dir).join(&db).exists());
    assert_eq!(
        server
            .delete(&format!("/db/{db}/delete"), &user.token)
            .await?,
        204
    );
    assert!(!Path::new(&server.data_dir).join(db).exists());
    Ok(())
}

#[tokio::test]
async fn in_memory_db() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let db = server.init_db("memory", &user).await?;
    assert_eq!(
        server
            .delete(&format!("/db/{db}/delete"), &user.token)
            .await?,
        204
    );
    let (_, list) = server
        .get::<Vec<DbWithRole>>(DB_LIST_URI, &user.token)
        .await?;
    assert_eq!(list?, vec![]);
    Ok(())
}

#[tokio::test]
async fn db_not_found() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    assert_eq!(
        server
            .delete(&format!("/db/{}/not_found/delete", user.name), &user.token)
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
            .put(
                &format!("/db/{db}/user/{}/add?db_role=admin", other.name),
                &String::new(),
                &user.token
            )
            .await?,
        201
    );
    assert_eq!(
        server
            .delete(&format!("/db/{db}/delete"), &other.token)
            .await?,
        403
    );
    Ok(())
}

#[tokio::test]
async fn no_token() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    assert_eq!(server.delete("/db/user/db/delete", NO_TOKEN).await?, 401);
    Ok(())
}
