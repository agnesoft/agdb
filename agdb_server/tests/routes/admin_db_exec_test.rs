use crate::next_db_name;
use crate::next_user_name;
use crate::TestCluster;
use crate::TestServer;
use crate::ADMIN;
use agdb::DbElement;
use agdb::DbId;
use agdb::QueryBuilder;
use agdb::QueryResult;
use agdb_api::DbType;

#[tokio::test]
async fn read_write() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_db_add(owner, db, DbType::Mapped).await?;
    let queries = &vec![
        QueryBuilder::insert()
            .nodes()
            .aliases("root")
            .values([[("key", 1.1).into()]])
            .query()
            .into(),
        QueryBuilder::select().ids("root").query().into(),
    ];
    let (status, results) = server.api.admin_db_exec_mut(owner, db, queries).await?;
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
    let owner = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_db_add(owner, db, DbType::Mapped).await?;
    let queries = &vec![QueryBuilder::insert()
        .nodes()
        .aliases("root")
        .values([[("key", 1.1).into()]])
        .query()
        .into()];
    let (status, _) = server.api.admin_db_exec_mut(owner, db, queries).await?;
    assert_eq!(status, 200);
    let queries = &vec![QueryBuilder::select().ids("root").query().into()];
    let (status, results) = server.api.admin_db_exec(owner, db, queries).await?;
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
async fn query_error() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_db_add(owner, db, DbType::Mapped).await?;
    let queries = &vec![
        QueryBuilder::insert()
            .nodes()
            .values([[("key", 1.1).into()]])
            .query()
            .into(),
        QueryBuilder::select().ids("root").query().into(),
    ];
    let error = server
        .api
        .admin_db_exec_mut(owner, db, queries)
        .await
        .unwrap_err();
    assert_eq!(error.status, 470);
    assert_eq!(error.description, "Alias 'root' not found");
    Ok(())
}

#[tokio::test]
async fn db_not_found() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    server.api.user_login(ADMIN, ADMIN).await?;
    let status = server
        .api
        .admin_db_exec("owner", "db", &[])
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 404);
    Ok(())
}

#[tokio::test]
async fn non_admin() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.user_login(owner, owner).await?;
    let status = server
        .api
        .admin_db_exec(owner, "db", &[])
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 401);
    Ok(())
}

#[tokio::test]
async fn no_token() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let status = server
        .api
        .admin_db_exec("owner", "db", &[])
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 401);
    Ok(())
}

#[tokio::test]
async fn cluster_exec() -> anyhow::Result<()> {
    let mut cluster = TestCluster::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    let client = cluster.apis.get_mut(1).unwrap();
    client.user_login(ADMIN, ADMIN).await?;
    client.admin_user_add(owner, owner).await?;
    client.admin_db_add(owner, db, DbType::Memory).await?;
    client
        .admin_db_exec_mut(
            owner,
            db,
            &[QueryBuilder::insert()
                .nodes()
                .aliases("root")
                .query()
                .into()],
        )
        .await?;
    client.user_login(owner, owner).await?;
    let result = client
        .db_exec(
            owner,
            db,
            &[QueryBuilder::select().ids("root").query().into()],
        )
        .await?
        .1[0]
        .result;
    assert_eq!(result, 1);
    Ok(())
}
