use crate::TestServer;
use crate::NO_TOKEN;
use std::path::Path;

#[tokio::test]
async fn add() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let db = format!("{}/db_add_test", user.name);
    assert_eq!(
        server
            .post(
                &format!("/db/{db}/add?db_type=file"),
                &String::new(),
                &user.token
            )
            .await?
            .0,
        201
    );
    assert!(Path::new(&server.data_dir).join(db).exists());
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
            .post(
                &format!("/db/{db1}/add?db_type=file"),
                &String::new(),
                &user.token
            )
            .await?
            .0,
        201
    );
    assert!(Path::new(&server.data_dir).join(db1).exists());
    let db2 = format!("{}/add_same_name_different_user", other.name);
    assert_eq!(
        server
            .post(
                &format!("/db/{db2}/add?db_type=mapped"),
                &String::new(),
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
            .post(
                &format!("/db/{db}/add?db_type=file"),
                &String::new(),
                &user.token
            )
            .await?
            .0,
        201
    );
    assert_eq!(
        server
            .post(
                &format!("/db/{db}/add?db_type=file"),
                &String::new(),
                &user.token
            )
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
            .post(
                "/db/some_user/db/add?db_type=file",
                &String::new(),
                &user.token
            )
            .await?
            .0,
        403
    );
    Ok(())
}

#[tokio::test]
async fn no_token() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    assert_eq!(
        server
            .post(
                "/db/some_user/db/add?db_type=file",
                &String::new(),
                NO_TOKEN
            )
            .await?
            .0,
        401
    );
    Ok(())
}
