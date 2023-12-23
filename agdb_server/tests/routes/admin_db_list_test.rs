use crate::Db;
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
        .get::<Vec<Db>>(ADMIN_DB_LIST_URI, &server.admin_token)
        .await?;
    assert_eq!(status, 200);
    let list = list?;
    assert!(list.contains(&Db {
        name: db1,
        db_type: "memory".to_string(),
        role: "admin".to_string(),
        size: 2600,
        backup: 0,
    }));
    assert!(list.contains(&Db {
        name: db2,
        db_type: "memory".to_string(),
        role: "admin".to_string(),
        size: 2600,
        backup: 0,
    }));

    Ok(())
}

#[tokio::test]
async fn with_backup() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let db = server.init_db("mapped", &user).await?;
    server
        .post(&format!("/db/{db}/backup"), &String::new(), &user.token)
        .await?;
    let (status, list) = server
        .get::<Vec<Db>>(ADMIN_DB_LIST_URI, &server.admin_token)
        .await?;
    assert_eq!(status, 200);
    let list = list?;
    let db = list.iter().find(|d| d.name == db).unwrap();
    assert_ne!(db.backup, 0);

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
