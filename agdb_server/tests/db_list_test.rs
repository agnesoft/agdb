pub mod framework;

use crate::framework::Db;
use crate::framework::TestServer;
use crate::framework::DB_LIST_URI;
use crate::framework::NO_TOKEN;

#[tokio::test]
async fn list() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let token = server.init_user("alice", "password123").await?;
    server.init_db("my_db", "memory", &token).await?;
    server.init_db("my_other_db", "memory", &token).await?;
    let expected = vec![
        Db {
            name: "my_db".to_string(),
            db_type: "memory".to_string(),
        },
        Db {
            name: "my_other_db".to_string(),
            db_type: "memory".to_string(),
        },
    ];
    let (status, list) = server.get::<Vec<Db>>(DB_LIST_URI, &token).await?;
    assert_eq!(status, 200);
    let mut list = list?;
    list.sort();
    assert_eq!(list, expected);
    Ok(())
}

#[tokio::test]
async fn list_empty() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let token = server.init_user("alice", "password123").await?;
    let (status, list) = server.get::<Vec<Db>>(DB_LIST_URI, &token).await?;
    assert_eq!(status, 200);
    assert_eq!(list?, vec![]);
    Ok(())
}

#[tokio::test]
async fn list_no_token() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let (status, list) = server.get::<Vec<Db>>(DB_LIST_URI, NO_TOKEN).await?;
    assert_eq!(status, 401);
    assert!(list.is_err());
    Ok(())
}
