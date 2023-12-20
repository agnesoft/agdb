use crate::Db;
use crate::DbWithSize;
use crate::TestServer;
use crate::ADMIN_DB_LIST_URI;
use crate::NO_TOKEN;

#[tokio::test]
async fn db_list() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user1 = server.init_user().await?;
    let user2 = server.init_user().await?;
    let db1 = server.init_db("memory", &user1).await?;
    let db2 = server.init_db("memory", &user2).await?;
    let (status, list) = server
        .get::<Vec<DbWithSize>>(ADMIN_DB_LIST_URI, &server.admin_token)
        .await?;
    assert_eq!(status, 200);
    let list = list?;
    assert!(list.contains(&DbWithSize {
        name: db1,
        db_type: "memory".to_string(),
        size: 2600,
    }));
    assert!(list.contains(&DbWithSize {
        name: db2,
        db_type: "memory".to_string(),
        size: 2600,
    }));

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
