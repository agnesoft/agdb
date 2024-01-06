use crate::TestServer;
use crate::NO_TOKEN;
use agdb::DbElement;
use agdb::DbId;
use agdb::QueryBuilder;
use agdb::QueryResult;
use agdb::QueryType;
use agdb_api::DbType;

#[tokio::test]
async fn read_write() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let user = server.init_user().await?;
    let db = server.init_db(DbType::Memory, &user).await?;
    let queries: Option<Vec<QueryType>> = Some(vec![
        QueryBuilder::insert()
            .nodes()
            .aliases("root")
            .values(vec![vec![("key", 1.1).into()]])
            .query()
            .into(),
        QueryBuilder::select().ids("root").query().into(),
    ]);
    let (status, responses) = server
        .post(&format!("/db/{db}/exec"), &queries, &user.token)
        .await?;
    assert_eq!(status, 200);
    let responses: Vec<QueryResult> = serde_json::from_str(&responses)?;
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

    assert_eq!(responses.len(), expected.len());
    assert_eq!(responses[0].result, expected[0].result);
    assert_eq!(responses[0].elements, expected[0].elements);
    assert_eq!(responses[1].result, expected[1].result);
    assert_eq!(responses[1].elements, expected[1].elements);

    Ok(())
}

#[tokio::test]
async fn read_only() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let user = server.init_user().await?;
    let db = server.init_db(DbType::Memory, &user).await?;
    let queries: Option<Vec<QueryType>> = Some(vec![
        // Wrap the queries vector with Some()
        QueryBuilder::insert()
            .nodes()
            .aliases("root")
            .values(vec![vec![("key", 1.1).into()]])
            .query()
            .into(),
    ]);
    let (status, _responses) = server
        .post(&format!("/db/{db}/exec"), &queries, &user.token) // Pass queries as &queries instead of &queries
        .await?;
    assert_eq!(status, 200);
    let queries: Option<Vec<QueryType>> =
        Some(vec![QueryBuilder::select().ids("root").query().into()]);
    let (status, responses) = server
        .post(&format!("/db/{db}/exec"), &queries, &user.token)
        .await?;
    assert_eq!(status, 200);
    let responses: Vec<QueryResult> = serde_json::from_str(&responses)?;
    let expected = vec![QueryResult {
        result: 1,
        elements: vec![DbElement {
            id: DbId(1),
            from: None,
            to: None,
            values: vec![("key", 1.1).into()],
        }],
    }];

    assert_eq!(responses.len(), expected.len());
    assert_eq!(responses[0].result, expected[0].result);
    assert_eq!(responses[0].elements, expected[0].elements);

    Ok(())
}

#[tokio::test]
async fn read_queries() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let user = server.init_user().await?;
    let db = server.init_db(DbType::Memory, &user).await?;
    let queries: Option<Vec<QueryType>> = Some(vec![QueryBuilder::insert()
        .nodes()
        .aliases("node1")
        .values(vec![vec![("key", "value").into()]])
        .query()
        .into()]);
    server
        .post(&format!("/db/{db}/exec"), &queries, &user.token)
        .await?;
    let queries: Option<Vec<QueryType>> = Some(vec![
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
    ]);
    let (status, responses) = server
        .post(&format!("/db/{db}/exec"), &queries, &user.token)
        .await?;
    assert_eq!(status, 200);
    let responses: Vec<QueryResult> = serde_json::from_str(&responses)?;
    assert_eq!(responses.len(), 8);

    Ok(())
}

#[tokio::test]
async fn write_queries() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let user = server.init_user().await?;
    let db = server.init_db(DbType::Memory, &user).await?;
    let queries: Option<Vec<QueryType>> = Some(vec![
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
    ]);
    let (status, responses) = server
        .post(&format!("/db/{db}/exec"), &queries, &user.token)
        .await?;
    assert_eq!(status, 200);
    let responses: Vec<QueryResult> = serde_json::from_str(&responses)?;
    assert_eq!(responses.len(), 17);

    Ok(())
}

#[tokio::test]
async fn query_error() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let user = server.init_user().await?;
    let db = server.init_db(DbType::Memory, &user).await?;
    let queries: Option<Vec<QueryType>> = Some(vec![
        QueryBuilder::insert()
            .nodes()
            .aliases("")
            .values(vec![vec![("key", 1.1).into()]])
            .query()
            .into(),
        QueryBuilder::select().ids("root").query().into(),
    ]);
    let (status, response) = server
        .post(&format!("/db/{db}/exec"), &queries, &user.token)
        .await?;
    assert_eq!(status, 470);
    assert_eq!(response, "Alias 'root' not found");

    Ok(())
}

#[tokio::test]
async fn permission_denied() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let user = server.init_user().await?;
    let db = server.init_db(DbType::Memory, &user).await?;
    let other = server.init_user().await?;
    assert_eq!(
        server
            .put::<()>(
                &format!("/db/{db}/user/{}/add?db_role=read", other.name),
                &None,
                &user.token
            )
            .await?,
        201
    );
    let queries: Option<Vec<QueryType>> = Some(vec![
        QueryBuilder::insert()
            .nodes()
            .aliases("root")
            .values(vec![vec![("key", 1.1).into()]])
            .query()
            .into(),
        QueryBuilder::select().ids("root").query().into(),
    ]);
    let status = server
        .post(&format!("/db/{db}/exec"), &queries, &other.token)
        .await?
        .0;
    assert_eq!(status, 403);

    Ok(())
}

#[tokio::test]
async fn db_not_found() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let user = server.init_user().await?;
    let queries: Option<Vec<QueryType>> = Some(vec![]);
    let status = server
        .post("/db/user/not_found/exec", &queries, &user.token)
        .await?
        .0;
    assert_eq!(status, 404);

    Ok(())
}

#[tokio::test]
async fn no_token() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let queries: Option<Vec<QueryType>> = Some(vec![]);
    let status = server
        .post("/db/user/not_found/exec", &queries, NO_TOKEN)
        .await?
        .0;
    assert_eq!(status, 401);

    Ok(())
}
