use crate::Db;
use crate::TestServer;
use crate::DB_LIST_URI;
use crate::NO_TOKEN;
use std::path::Path;

#[tokio::test]
async fn add() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let name = format!("{}/admin_db_add_test", user.name);
    assert_eq!(
        server
            .post::<()>(
                &format!("/admin/db/{name}/add?db_type=file"),
                &None,
                &server.admin_token
            )
            .await?
            .0,
        201
    );
    assert!(Path::new(&server.data_dir).join(name).exists());
    Ok(())
}

#[tokio::test]
async fn add_with_backup() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let db = server.init_db("mapped", &user).await?;
    server
        .post::<()>(&format!("/db/{db}/backup"), &None, &user.token)
        .await?;
    server
        .delete(&format!("/db/{db}/remove"), &user.token)
        .await?;
    assert_eq!(
        server
            .post::<()>(
                &format!("/admin/db/{db}/add?db_type=mapped"),
                &None,
                &server.admin_token
            )
            .await?
            .0,
        201
    );
    let (status, list) = server.get::<Vec<Db>>(DB_LIST_URI, &user.token).await?;
    assert_eq!(status, 200);
    assert_ne!(list?[0].backup, 0);
    Ok(())
}

#[tokio::test]
async fn db_already_exists() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let name = format!("{}/mydb", user.name);
    assert_eq!(
        server
            .post::<()>(
                &format!("/admin/db/{name}/add?db_type=memory"),
                &None,
                &server.admin_token
            )
            .await?
            .0,
        201
    );
    assert_eq!(
        server
            .post::<()>(
                &format!("/admin/db/{name}/add?db_type=memory"),
                &None,
                &server.admin_token
            )
            .await?
            .0,
        465
    );
    Ok(())
}

#[tokio::test]
async fn user_not_found() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    assert_eq!(
        server
            .post::<()>(
                "/admin/db/not_found/admin_db_add_test/add?db_type=mapped",
                &None,
                &server.admin_token
            )
            .await?
            .0,
        404
    );
    Ok(())
}

#[tokio::test]
async fn non_admin() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let name = format!("{}/admin_db_add_test", user.name);
    assert_eq!(
        server
            .post::<()>(
                &format!("/admin/db/{name}/add?db_type=file"),
                &None,
                &user.token
            )
            .await?
            .0,
        401
    );
    Ok(())
}

#[tokio::test]
async fn no_token() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    assert_eq!(
        server
            .post::<()>(
                "/admin/db/not_found/admin_db_add_test/add?db_type=mapped",
                &None,
                NO_TOKEN
            )
            .await?
            .0,
        401
    );
    Ok(())
}
