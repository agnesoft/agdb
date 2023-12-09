pub mod framework;

use crate::framework::Db;
use crate::framework::TestServer;
use crate::framework::DB_ADD_URI;
use crate::framework::DB_LIST_URI;

#[tokio::test]
async fn list() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let token = server.init_user("alice", "mypassword123").await?;
    let bad = Some("bad".to_string());
    let db1 = Db {
        name: "db1".to_string(),
        db_type: "memory".to_string(),
    };
    let db2 = Db {
        name: "db2".to_string(),
        db_type: "memory".to_string(),
    };

    assert_eq!(server.get::<()>(DB_LIST_URI, &bad).await?.0, 401); //unauthorized
    let (status, list) = server.get::<Vec<Db>>(DB_LIST_URI, &token).await?;
    assert_eq!(status, 200); //Ok
    assert_eq!(list?, vec![]);
    assert_eq!(server.post(DB_ADD_URI, &db1, &token).await?.0, 201); //created
    assert_eq!(server.post(DB_ADD_URI, &db2, &token).await?.0, 201); //created
    let (status, list) = server.get::<Vec<Db>>(DB_LIST_URI, &token).await?;
    let mut list = list?;
    list.sort();
    assert_eq!(status, 200); //Ok
    assert_eq!(list, vec![db1, db2]);

    Ok(())
}
