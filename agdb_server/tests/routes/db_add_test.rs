use crate::framework::Db;
use crate::framework::TestServer;
use crate::framework::DB_ADD_URI;
use crate::framework::NO_TOKEN;
use std::path::Path;

#[tokio::test]
async fn add() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let (_, token) = server.init_user().await?;
    let db = Db {
        name: "db_add_test".to_string(),
        db_type: "file".to_string(),
    };
    assert_eq!(server.post(DB_ADD_URI, &db, &token).await?.0, 201);
    assert!(Path::new(&server.dir).join(db.name).exists());
    Ok(())
}

#[tokio::test]
async fn db_already_exists() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let (_, token) = server.init_user().await?;
    let db = Db {
        name: "mydb".to_string(),
        db_type: "memory".to_string(),
    };
    assert_eq!(server.post(DB_ADD_URI, &db, &token).await?.0, 201);
    assert_eq!(server.post(DB_ADD_URI, &db, &token).await?.0, 465);
    Ok(())
}

#[tokio::test]
async fn db_invalid() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let (_, token) = server.init_user().await?;
    let db = Db {
        name: "".to_string(),
        db_type: "mapped".to_string(),
    };
    assert_eq!(server.post(DB_ADD_URI, &db, &token).await?.0, 467);
    Ok(())
}

#[tokio::test]
async fn no_token() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let db = Db {
        name: "mydb".to_string(),
        db_type: "mapped".to_string(),
    };
    assert_eq!(server.post(DB_ADD_URI, &db, NO_TOKEN).await?.0, 401);
    Ok(())
}
