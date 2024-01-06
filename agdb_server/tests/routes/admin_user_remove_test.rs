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

    assert_eq!(
        server
            .delete(
                &format!("/admin/user/{}/remove", user.name),
                &server.admin_token
            )
            .await?,
        204
    );

    Ok(())
}

#[tokio::test]
async fn remove_with_other() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let user = server.init_user().await?;
    let other = server.init_user().await?;
    let db = server.init_db(DbType::Mapped, &user).await?;

    assert_eq!(
        server
            .put::<()>(
                &format!("/admin/db/{db}/user/{}/add?db_role=write", other.name),
                &None,
                &server.admin_token
            )
            .await?,
        201
    );

    let list = server.get::<Vec<Db>>(DB_LIST_URI, &other.token).await?.1;
    assert_eq!(
        list?,
        vec![Db {
            name: db,
            db_type: "mapped".to_string(),
            role: "write".to_string(),
            size: 2632,
            backup: 0,
        }]
    );

    assert!(Path::new(&server.data_dir).join(&user.name).exists());

    assert_eq!(
        server
            .delete(
                &format!("/admin/user/{}/remove", user.name),
                &server.admin_token
            )
            .await?,
        204
    );

    assert!(!Path::new(&server.data_dir).join(user.name).exists());

    let list = server.get::<Vec<Db>>(DB_LIST_URI, &other.token).await?.1;
    assert_eq!(list?, vec![]);

    Ok(())
}

#[tokio::test]
async fn user_not_found() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;

    assert_eq!(
        server
            .delete("/admin/user/not_found/remove", &server.admin_token)
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
            .delete("/admin/user/not_found/remove", &user.token)
            .await?,
        401
    );

    Ok(())
}

#[tokio::test]
async fn no_token() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;

    assert_eq!(
        server
            .delete("/admin/user/not_found/remove", NO_TOKEN)
            .await?,
        401
    );

    Ok(())
}
