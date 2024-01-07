use crate::TestServer;
use crate::ADMIN;
use agdb::DbElement;
use agdb::DbId;
use agdb::QueryBuilder;
use agdb::QueryResult;
use agdb_api::DbType;
use agdb_api::DbUserRole;

#[tokio::test]
async fn read_write() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &server.next_user_name();
    let db = &server.next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.user_login(owner, owner).await?;
    server.api.db_add(owner, db, DbType::Mapped).await?;
    let queries = &vec![
        QueryBuilder::insert()
            .nodes()
            .aliases("root")
            .values(vec![vec![("key", 1.1).into()]])
            .query()
            .into(),
        QueryBuilder::select().ids("root").query().into(),
    ];
    let (status, results) = server.api.db_exec(owner, db, queries).await?;
    assert_eq!(status, 200);
    let expected = vec![
        QueryResult {
            result: 1,
            elements: vec![DbElement {
                id: DbId(1),
                from: None,
                to: None,
                values: vec![],
            }],
        },
        QueryResult {
            result: 1,
            elements: vec![DbElement {
                id: DbId(1),
                from: None,
                to: None,
                values: vec![("key", 1.1).into()],
            }],
        },
    ];
    assert_eq!(results, expected);
    Ok(())
}

#[tokio::test]
async fn read_only() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &server.next_user_name();
    let db = &server.next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.user_login(owner, owner).await?;
    server.api.db_add(owner, db, DbType::Mapped).await?;
    let queries = &vec![QueryBuilder::insert()
        .nodes()
        .aliases("root")
        .values(vec![vec![("key", 1.1).into()]])
        .query()
        .into()];
    let (status, _) = server.api.db_exec(owner, db, queries).await?;
    assert_eq!(status, 200);
    let queries = &vec![QueryBuilder::select().ids("root").query().into()];
    let (status, results) = server.api.db_exec(owner, db, queries).await?;
    assert_eq!(status, 200);
    let expected = vec![QueryResult {
        result: 1,
        elements: vec![DbElement {
            id: DbId(1),
            from: None,
            to: None,
            values: vec![("key", 1.1).into()],
        }],
    }];
    assert_eq!(results, expected);
    Ok(())
}

#[tokio::test]
async fn read_queries() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &server.next_user_name();
    let db = &server.next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.user_login(owner, owner).await?;
    server.api.db_add(owner, db, DbType::Mapped).await?;
    let queries = &vec![QueryBuilder::insert()
        .nodes()
        .aliases("node1")
        .values(vec![vec![("key", "value").into()]])
        .query()
        .into()];
    server.api.db_exec(owner, db, queries).await?;
    let queries = &vec![
        QueryBuilder::search().from(1).query().into(),
        QueryBuilder::select().ids(1).query().into(),
        QueryBuilder::select().aliases().ids(1).query().into(),
        QueryBuilder::select().aliases().query().into(),
        QueryBuilder::select().indexes().query().into(),
        QueryBuilder::select().keys().ids(1).query().into(),
        QueryBuilder::select().key_count().ids(1).query().into(),
        QueryBuilder::select()
            .values(vec!["key".into()])
            .ids(1)
            .query()
            .into(),
    ];
    let (status, results) = server.api.db_exec(owner, db, queries).await?;
    assert_eq!(status, 200);
    assert_eq!(results.len(), 8);
    Ok(())
}

#[tokio::test]
async fn write_queries() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &server.next_user_name();
    let db = &server.next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.user_login(owner, owner).await?;
    server.api.db_add(owner, db, DbType::Mapped).await?;
    let queries = &vec![
        QueryBuilder::insert().nodes().count(2).query().into(),
        QueryBuilder::insert()
            .aliases(vec!["node1", "node2"])
            .ids(vec![1, 2])
            .query()
            .into(),
        QueryBuilder::insert()
            .edges()
            .from("node1")
            .to("node2")
            .query()
            .into(),
        QueryBuilder::insert()
            .values(vec![vec![("key", 1.1).into()]])
            .ids("node1")
            .query()
            .into(),
        QueryBuilder::insert().index("key").query().into(),
        QueryBuilder::search().from(1).query().into(),
        QueryBuilder::select().ids(1).query().into(),
        QueryBuilder::select().aliases().ids(1).query().into(),
        QueryBuilder::select().aliases().query().into(),
        QueryBuilder::select().indexes().query().into(),
        QueryBuilder::select().keys().ids(1).query().into(),
        QueryBuilder::select().key_count().ids(1).query().into(),
        QueryBuilder::select()
            .values(vec!["key".into()])
            .ids(1)
            .query()
            .into(),
        QueryBuilder::remove().aliases("node2").query().into(),
        QueryBuilder::remove().index("key").query().into(),
        QueryBuilder::remove()
            .values(vec!["key".into()])
            .ids(1)
            .query()
            .into(),
        QueryBuilder::remove().ids("node1").query().into(),
    ];
    let (status, results) = server.api.db_exec(owner, db, queries).await?;
    assert_eq!(status, 200);
    assert_eq!(results.len(), 17);
    Ok(())
}

#[tokio::test]
async fn query_error() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &server.next_user_name();
    let db = &server.next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.user_login(owner, owner).await?;
    server.api.db_add(owner, db, DbType::Mapped).await?;
    let queries = &vec![
        QueryBuilder::insert()
            .nodes()
            .values(vec![vec![("key", 1.1).into()]])
            .query()
            .into(),
        QueryBuilder::select().ids("root").query().into(),
    ];
    let error = server.api.db_exec(owner, db, queries).await.unwrap_err();
    assert_eq!(error.status, 470);
    assert_eq!(error.description, "Alias 'root' not found");
    Ok(())
}

#[tokio::test]
async fn permission_denied() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &server.next_user_name();
    let user = &server.next_user_name();
    let db = &server.next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_user_add(user, user).await?;
    server.api.admin_db_add(owner, db, DbType::Mapped).await?;
    server
        .api
        .admin_db_user_add(owner, db, user, DbUserRole::Read)
        .await?;
    server.api.user_login(user, user).await?;
    let queries = &vec![
        QueryBuilder::insert()
            .nodes()
            .aliases("root")
            .values(vec![vec![("key", 1.1).into()]])
            .query()
            .into(),
        QueryBuilder::select().ids("root").query().into(),
    ];
    let error = server.api.db_exec(owner, db, queries).await.unwrap_err();
    assert_eq!(error.status, 403);
    Ok(())
}

#[tokio::test]
async fn db_not_found() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &server.next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.user_login(owner, owner).await?;
    let status = server
        .api
        .db_exec(owner, "db", &vec![])
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 404);
    Ok(())
}

#[tokio::test]
async fn no_token() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let status = server
        .api
        .db_exec("user", "db", &vec![])
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 401);
    Ok(())
}
