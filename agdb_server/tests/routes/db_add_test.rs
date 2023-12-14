use crate::Db;
use crate::TestServer;
use crate::DB_ADD_URI;
use crate::NO_TOKEN;
use std::path::Path;

#[tokio::test]
async fn add() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let db = Db {
        name: format!("{}/db_add_test", user.name),
        db_type: "file".to_string(),
    };
    assert_eq!(server.post(DB_ADD_URI, &db, &user.token).await?.0, 201);
    assert!(Path::new(&server.dir).join(db.name).exists());
    Ok(())
}

#[tokio::test]
async fn db_already_exists() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let db = Db {
        name: format!("{}/mydb", user.name),
        db_type: "memory".to_string(),
    };
    assert_eq!(server.post(DB_ADD_URI, &db, &user.token).await?.0, 201);
    assert_eq!(server.post(DB_ADD_URI, &db, &user.token).await?.0, 465);
    Ok(())
}

#[tokio::test]
async fn db_user_mismatch() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let other = server.init_user().await?;
    let db = Db {
        name: format!("{}/", other.name),
        db_type: "mapped".to_string(),
    };
    assert_eq!(server.post(DB_ADD_URI, &db, &user.token).await?.0, 464);
    Ok(())
}

#[tokio::test]
async fn db_invalid() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let db = Db {
        name: format!("{}/", user.name),
        db_type: "mapped".to_string(),
    };
    assert_eq!(server.post(DB_ADD_URI, &db, &user.token).await?.0, 467);
    Ok(())
}

#[tokio::test]
async fn no_token() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let db = Db {
        name: "user/mydb".to_string(),
        db_type: "mapped".to_string(),
    };
    assert_eq!(server.post(DB_ADD_URI, &db, NO_TOKEN).await?.0, 401);
    Ok(())
}
