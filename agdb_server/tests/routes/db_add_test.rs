use crate::Db;
use crate::TestServer;
use crate::DB_LIST_URI;
use crate::NO_TOKEN;
use std::path::Path;

#[tokio::test]
async fn add() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let db = format!("{}/db_add_test", user.name);
    assert_eq!(
        server
            .post::<()>(&format!("/db/{db}/add?db_type=file"), &None, &user.token)
            .await?
            .0,
        201
    );
    assert!(Path::new(&server.data_dir).join(db).exists());
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
            .post::<()>(&format!("/db/{db}/add?db_type=mapped"), &None, &user.token)
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
async fn add_same_name_different_user() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let other = server.init_user().await?;
    let db1 = format!("{}/add_same_name_different_user", user.name);
    assert_eq!(
        server
            .post::<()>(&format!("/db/{db1}/add?db_type=file"), &None, &user.token)
            .await?
            .0,
        201
    );
    assert!(Path::new(&server.data_dir).join(db1).exists());
    let db2 = format!("{}/add_same_name_different_user", other.name);
    assert_eq!(
        server
            .post::<()>(
                &format!("/db/{db2}/add?db_type=mapped"),
                &None,
                &other.token
            )
            .await?
            .0,
        201
    );
    assert!(Path::new(&server.data_dir).join(db2).exists());
    Ok(())
}

#[tokio::test]
async fn db_already_exists() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let db = format!("{}/db_add_test", user.name);
    assert_eq!(
        server
            .post::<()>(&format!("/db/{db}/add?db_type=file"), &None, &user.token)
            .await?
            .0,
        201
    );
    assert_eq!(
        server
            .post::<()>(&format!("/db/{db}/add?db_type=file"), &None, &user.token)
            .await?
            .0,
        465
    );
    Ok(())
}

#[tokio::test]
async fn db_user_mismatch() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    assert_eq!(
        server
            .post::<()>("/db/some_user/db/add?db_type=file", &None, &user.token)
            .await?
            .0,
        403
    );
    Ok(())
}

#[tokio::test]
async fn db_type_invalid() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    assert_eq!(
        server
            .post::<()>(
                &format!("/db/{}/a\0a/add?db_type=file", user.name),
                &None,
                &user.token
            )
            .await?
            .0,
        467
    );
    Ok(())
}

#[tokio::test]
async fn no_token() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    assert_eq!(
        server
            .post::<()>("/db/some_user/db/add?db_type=file", &None, NO_TOKEN)
            .await?
            .0,
        401
    );
    Ok(())
}
