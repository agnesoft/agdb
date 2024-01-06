use crate::TestServer;
use crate::NO_TOKEN;
use agdb_api::DbType;
use std::path::Path;

#[tokio::test]
async fn delete() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let user = server.init_user().await?;
    let db = server.init_db(DbType::Mapped, &user).await?;
    assert!(Path::new(&server.data_dir).join(&db).exists());
    assert_eq!(
        server
            .delete(&format!("/admin/db/{db}/delete"), &server.admin_token)
            .await?,
        204
    );
    assert!(!Path::new(&server.data_dir).join(db).exists());
    Ok(())
}

#[tokio::test]
async fn delete_with_backup() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let user = server.init_user().await?;
    let db = server.init_db(DbType::Mapped, &user).await?;
    server
        .post::<()>(&format!("/db/{db}/backup"), &None, &user.token)
        .await?;
    assert!(Path::new(&server.data_dir)
        .join(&user.name)
        .join("backups")
        .join(format!("{}.bak", db.split_once('/').unwrap().1))
        .exists());
    assert!(Path::new(&server.data_dir).join(&db).exists());
    assert_eq!(
        server
            .delete(&format!("/admin/db/{db}/delete"), &server.admin_token)
            .await?,
        204
    );
    assert!(!Path::new(&server.data_dir).join(&db).exists());
    assert!(!Path::new(&server.data_dir)
        .join(user.name)
        .join("backups")
        .join(format!("{}.bak", db.split_once('/').unwrap().1))
        .exists());
    Ok(())
}

#[tokio::test]
async fn db_not_found() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let user = server.init_user().await?;
    assert_eq!(
        server
            .delete(
                &format!("/admin/db/{}/not_found/delete", user.name),
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
            .delete("/admin/db/not_found/db/delete", &server.admin_token)
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
            .delete("/admin/db/user/db/delete", &user.token)
            .await?,
        401
    );
    Ok(())
}

#[tokio::test]
async fn no_token() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    assert_eq!(
        server.delete("/admin/db/user/db/delete", NO_TOKEN).await?,
        401
    );
    Ok(())
}
