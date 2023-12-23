use crate::Db;
use crate::TestServer;
use crate::DB_LIST_URI;
use crate::NO_TOKEN;

#[tokio::test]
async fn list() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let db1 = server.init_db("memory", &user).await?;
    let db2 = server.init_db("mapped", &user).await?;
    let mut expected = vec![
        Db {
            name: db1.clone(),
            db_type: "memory".to_string(),
            role: "admin".to_string(),
            size: 2600,
            backup: 0,
        },
        Db {
            name: db2.clone(),
            db_type: "mapped".to_string(),
            role: "admin".to_string(),
            size: 2600,
            backup: 0,
        },
    ];
    expected.sort();
    let (status, list) = server.get::<Vec<Db>>(DB_LIST_URI, &user.token).await?;
    assert_eq!(status, 200);
    let mut list = list?;
    list.sort();
    assert_eq!(list, expected);
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
    let (status, list) = server.get::<Vec<Db>>(DB_LIST_URI, &user.token).await?;
    assert_eq!(status, 200);
    assert_ne!(list?[0].backup, 0);

    Ok(())
}

#[tokio::test]
async fn list_empty() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let (status, list) = server.get::<Vec<Db>>(DB_LIST_URI, &user.token).await?;
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
