pub mod framework;

use crate::framework::AddUser;
use crate::framework::Db;
use crate::framework::TestServer;
use crate::framework::DB_ADD_URI;
use crate::framework::DB_LIST_URI;
use crate::framework::DB_USER_ADD_URI;

#[tokio::test]
async fn add_user() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let token = server.init_user("alice", "mypassword123").await?;
    let token2: Option<String> = server.init_user("bob", "mypassword456").await?;
    server.init_user("chad", "mypassword789").await?;
    let bad_token = Some("bad".to_string());
    let db = Db {
        name: "db1".to_string(),
        db_type: "mapped".to_string(),
    };
    let add_read = AddUser {
        database: "db1",
        user: "bob",
        role: "read",
    };
    let add_write = AddUser {
        database: "db1",
        user: "bob",
        role: "write",
    };
    let add_chad = AddUser {
        database: "db1",
        user: "chad",
        role: "read",
    };
    let add_admin = AddUser {
        database: "db1",
        user: "bob",
        role: "admin",
    };
    let add_self = AddUser {
        database: "db1",
        user: "alice",
        role: "read",
    };
    let no_user = AddUser {
        database: "db1",
        user: "cat",
        role: "read",
    };
    let no_db = AddUser {
        database: "db_missing",
        user: "bob",
        role: "read",
    };
    assert_eq!(server.post(DB_ADD_URI, &db, &token).await?.0, 201); //created
    let (status, list) = server.get::<Vec<Db>>(DB_LIST_URI, &token2).await?;
    assert_eq!(status, 200); //Ok
    let list = list?;
    assert_eq!(list, vec![]);
    assert_eq!(
        server.post(DB_USER_ADD_URI, &add_read, &bad_token).await?.0,
        401
    ); //forbidden
    assert_eq!(server.post(DB_USER_ADD_URI, &no_db, &token).await?.0, 466); //missing db
    assert_eq!(server.post(DB_USER_ADD_URI, &no_user, &token).await?.0, 464); //missing user
    assert_eq!(
        server.post(DB_USER_ADD_URI, &add_self, &token).await?.0,
        403
    ); //self
    assert_eq!(
        server.post(DB_USER_ADD_URI, &add_read, &token).await?.0,
        201
    ); //created
    assert_eq!(
        server.post(DB_USER_ADD_URI, &add_chad, &token2).await?.0,
        403
    ); //not an admin
    assert_eq!(
        server.post(DB_USER_ADD_URI, &add_write, &token).await?.0,
        201
    ); //created
    assert_eq!(
        server.post(DB_USER_ADD_URI, &add_chad, &token2).await?.0,
        403
    ); //not an admin
    assert_eq!(
        server.post(DB_USER_ADD_URI, &add_admin, &token).await?.0,
        201
    ); //created
    let (status, list) = server.get::<Vec<Db>>(DB_LIST_URI, &token2).await?;
    let list = list?;
    assert_eq!(status, 200); //Ok
    assert_eq!(list, vec![db]);
    assert_eq!(
        server.post(DB_USER_ADD_URI, &add_chad, &token2).await?.0,
        201
    ); //created

    Ok(())
}
