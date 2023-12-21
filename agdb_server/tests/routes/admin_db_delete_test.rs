use crate::TestServer;
use crate::NO_TOKEN;
use std::path::Path;

#[tokio::test]
async fn delete() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let db = server.init_db("mapped", &user).await?;
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
async fn db_not_found() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
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
    let server = TestServer::new().await?;
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
    let server = TestServer::new().await?;
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
    let server = TestServer::new().await?;
    assert_eq!(
        server.delete("/admin/db/user/db/delete", NO_TOKEN).await?,
        401
    );
    Ok(())
}
