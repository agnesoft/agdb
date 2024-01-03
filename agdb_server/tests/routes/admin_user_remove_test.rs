use crate::Db;
use crate::TestServer;
use crate::DB_LIST_URI;
use crate::NO_TOKEN;
use std::path::Path;

#[tokio::test]
async fn remove() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;

    assert_eq!(
        server
            .post::<()>(
                &format!("/admin/user/{}/remove", user.name),
                &None,
                &server.admin_token
            )
            .await?
            .0,
        204
    );

    Ok(())
}

#[tokio::test]
async fn remove_with_other() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let other = server.init_user().await?;
    let db = server.init_db("mapped", &user).await?;

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
            .post::<()>(
                &format!("/admin/user/{}/remove", user.name),
                &None,
                &server.admin_token
            )
            .await?
            .0,
        204
    );

    assert!(!Path::new(&server.data_dir).join(user.name).exists());

    let list = server.get::<Vec<Db>>(DB_LIST_URI, &other.token).await?.1;
    assert_eq!(list?, vec![]);

    Ok(())
}

#[tokio::test]
async fn user_not_found() -> anyhow::Result<()> {
    let server = TestServer::new().await?;

    assert_eq!(
        server
            .post::<()>("/admin/user/not_found/remove", &None, &server.admin_token)
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

    assert_eq!(
        server
            .post::<()>("/admin/user/not_found/remove", &None, &user.token)
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
            .post::<()>("/admin/user/not_found/remove", &None, NO_TOKEN)
            .await?
            .0,
        401
    );

    Ok(())
}
