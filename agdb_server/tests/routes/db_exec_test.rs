use crate::AddUser;
use crate::TestServer;
use crate::DB_EXEC_URI;
use crate::DB_USER_ADD_URI;
use crate::NO_TOKEN;
use agdb::DbElement;
use agdb::DbId;
use agdb::QueryBuilder;
use agdb::QueryResult;
use agdb::QueryType;

#[tokio::test]
async fn read_write() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let db = server.init_db("memory", &user).await?;
    let queries: Vec<QueryType> = vec![
        QueryBuilder::insert()
            .nodes()
            .aliases("root")
            .values(vec![vec![("key", 1.1).into()]])
            .query()
            .into(),
        QueryBuilder::select().ids("root").query().into(),
    ];
    let responses = server.exec(&db, &queries, &user.token).await?;
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
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let db = server.init_db("memory", &user).await?;
    let queries: Vec<QueryType> = vec![QueryBuilder::insert()
        .nodes()
        .aliases("root")
        .values(vec![vec![("key", 1.1).into()]])
        .query()
        .into()];
    server.exec(&db, &queries, &user.token).await?;
    let responses = server
        .exec(
            &db,
            &vec![QueryBuilder::select().ids("root").query().into()],
            &user.token,
        )
        .await?;
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
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let db = server.init_db("memory", &user).await?;
    let queries: Vec<QueryType> = vec![QueryBuilder::insert()
        .nodes()
        .aliases("node1")
        .values(vec![vec![("key", "value").into()]])
        .query()
        .into()];
    server.exec(&db, &queries, &user.token).await?;
    let queries: Vec<QueryType> = vec![
        QueryBuilder::search().from(1).query().into(),
        QueryBuilder::select().ids(1).query().into(),
        QueryBuilder::select().aliases().ids(1).query().into(),
        QueryBuilder::select().aliases().query().into(),
        QueryBuilder::select().keys().ids(1).query().into(),
        QueryBuilder::select().key_count().ids(1).query().into(),
        QueryBuilder::select()
            .values(vec!["key".into()])
            .ids(1)
            .query()
            .into(),
    ];
    let responses = server.exec(&db, &queries, &user.token).await?;
    assert_eq!(responses.len(), 7);

    Ok(())
}

#[tokio::test]
async fn write_queries() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let db = server.init_db("memory", &user).await?;
    let queries: Vec<QueryType> = vec![
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
        QueryBuilder::search().from(1).query().into(),
        QueryBuilder::select().ids(1).query().into(),
        QueryBuilder::select().aliases().ids(1).query().into(),
        QueryBuilder::select().aliases().query().into(),
        QueryBuilder::select().keys().ids(1).query().into(),
        QueryBuilder::select().key_count().ids(1).query().into(),
        QueryBuilder::select()
            .values(vec!["key".into()])
            .ids(1)
            .query()
            .into(),
        QueryBuilder::remove().aliases("node2").query().into(),
        QueryBuilder::remove()
            .values(vec!["key".into()])
            .ids(1)
            .query()
            .into(),
        QueryBuilder::remove().ids("node1").query().into(),
    ];
    let responses = server.exec(&db, &queries, &user.token).await?;
    assert_eq!(responses.len(), 14);

    Ok(())
}

#[tokio::test]
async fn query_error() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let db = server.init_db("memory", &user).await?;
    let queries: Vec<QueryType> = vec![
        QueryBuilder::insert()
            .nodes()
            .aliases("")
            .values(vec![vec![("key", 1.1).into()]])
            .query()
            .into(),
        QueryBuilder::select().ids("root").query().into(),
    ];
    let (status, response) = server
        .post(&format!("{DB_EXEC_URI}?db={db}"), &queries, &user.token)
        .await?;
    assert_eq!(status, 470);
    assert_eq!(response, "Alias 'root' not found");

    Ok(())
}

#[tokio::test]
async fn permission_denied() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let other = server.init_user().await?;
    let db = server.init_db("memory", &user).await?;
    let role = AddUser {
        database: &db,
        user: &other.name,
        role: "read",
    };

    server.post(DB_USER_ADD_URI, &role, &user.token).await?;

    let queries: Vec<QueryType> = vec![
        QueryBuilder::insert()
            .nodes()
            .aliases("root")
            .values(vec![vec![("key", 1.1).into()]])
            .query()
            .into(),
        QueryBuilder::select().ids("root").query().into(),
    ];
    let (status, _response) = server
        .post(&format!("{DB_EXEC_URI}?db={db}"), &queries, &other.token)
        .await?;
    assert_eq!(status, 403);

    Ok(())
}

#[tokio::test]
async fn db_not_found() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let queries: Vec<QueryType> = vec![];
    let status = server
        .post(
            &format!("{DB_EXEC_URI}?db={}/not_found", user.name),
            &queries,
            &user.token,
        )
        .await?
        .0;
    assert_eq!(status, 466);

    Ok(())
}

#[tokio::test]
async fn no_token() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let queries: Vec<QueryType> = vec![];
    let status = server
        .post(
            &format!("{DB_EXEC_URI}?db=user/not_found"),
            &queries,
            NO_TOKEN,
        )
        .await?
        .0;
    assert_eq!(status, 401);

    Ok(())
}
