use crate::Db;
use crate::TestServer;
use crate::DB_LIST_URI;
use crate::NO_TOKEN;
use agdb_api::DbType;
use std::path::Path;

#[tokio::test]
async fn remove() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let user = server.init_user().await?;
    let db = server.init_db(DbType::Mapped, &user).await?;
    assert_eq!(
        server
            .delete(&format!("/admin/db/{db}/remove"), &server.admin_token)
            .await?,
        204
    );
    let (status, list) = server.get::<Vec<Db>>(DB_LIST_URI, &user.token).await?;
    assert_eq!(status, 200);
    assert!(list?.is_empty());
    assert!(Path::new(&server.data_dir).join(db).exists());
    Ok(())
}

#[tokio::test]
async fn db_not_found() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let user = server.init_user().await?;
    assert_eq!(
        server
            .delete(
                &format!("/admin/db/{}/not_found/remove", user.name),
                &server.admin_token
            )
            .await?,
        404
    );
    Ok(())
}

#[tokio::test]
async fn user_not_found() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    assert_eq!(
        server
            .delete("/admin/db/not_found/db/remove", &server.admin_token)
            .await?,
        404
    );
    Ok(())
}

#[tokio::test]
async fn non_admin() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let user = server.init_user().await?;
    assert_eq!(
        server
            .delete("/admin/db/user/db/remove", &user.token)
            .await?,
        401
    );
    Ok(())
}

#[tokio::test]
async fn no_token() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    assert_eq!(
        server.delete("/admin/db/user/db/remove", NO_TOKEN).await?,
        401
    );
    Ok(())
}
