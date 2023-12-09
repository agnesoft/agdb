pub mod framework;

use crate::framework::Db;
use crate::framework::TestServer;
use crate::framework::ADMIN_DB_LIST_URI;
use crate::framework::NO_TOKEN;

#[tokio::test]
async fn db_list() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let user1 = server.init_user("alice", "password123").await?;
    let user2 = server.init_user("bob", "password456").await?;
    server.init_db("db1", "memory", &user1).await?;
    server.init_db("db2", "memory", &user2).await?;
    let (status, list) = server
        .get::<Vec<Db>>(ADMIN_DB_LIST_URI, &server.admin_token)
        .await?;
    assert_eq!(status, 200);
    let mut list = list?;
    list.sort();
    let expected = vec![
        Db {
            name: "db1".to_string(),
            db_type: "memory".to_string(),
        },
        Db {
            name: "db2".to_string(),
            db_type: "memory".to_string(),
        },
    ];
    assert_eq!(list, expected);
    Ok(())
}

#[tokio::test]
async fn db_not_found() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let admin = server.init_admin().await?;
    let (status, list) = server.get::<Vec<Db>>(ADMIN_DB_LIST_URI, &admin).await?;
    assert_eq!(status, 200);
    assert!(list?.is_empty());
    Ok(())
}

#[tokio::test]
async fn no_admin_token() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let (status, list) = server.get::<Vec<Db>>(ADMIN_DB_LIST_URI, NO_TOKEN).await?;
    assert_eq!(status, 401);
    assert!(list.is_err());
    Ok(())
}
