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
        .post(
            &format!("/admin/db/{db}/exec"),
            &queries,
            &server.admin_token,
        )
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
    let queries: Option<Vec<QueryType>> = Some(vec![QueryBuilder::insert()
        .nodes()
        .aliases("root")
        .values(vec![vec![("key", 1.1).into()]])
        .query()
        .into()]);
    let (status, _responses) = server
        .post(
            &format!("/admin/db/{db}/exec"),
            &queries,
            &server.admin_token,
        )
        .await?;
    assert_eq!(status, 200);
    let queries: Option<Vec<QueryType>> =
        Some(vec![QueryBuilder::select().ids("root").query().into()]);
    let (status, responses) = server
        .post(
            &format!("/admin/db/{db}/exec"),
            &queries,
            &server.admin_token,
        )
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
        .post(
            &format!("/admin/db/{db}/exec"),
            &queries,
            &server.admin_token,
        )
        .await?;
    assert_eq!(status, 470);
    assert_eq!(response, "Alias 'root' not found");

    Ok(())
}

#[tokio::test]
async fn db_not_found() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let queries: Option<Vec<QueryType>> = Some(vec![]);
    let status = server
        .post("/admin/db/user/db/exec", &queries, &server.admin_token)
        .await?
        .0;
    assert_eq!(status, 404);

    Ok(())
}

#[tokio::test]
async fn non_admin() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let user = server.init_user().await?;
    let queries: Option<Vec<QueryType>> = Some(vec![]);
    let status = server
        .post("/admin/db/user/db/exec", &queries, &user.token)
        .await?
        .0;
    assert_eq!(status, 401);

    Ok(())
}

#[tokio::test]
async fn no_token() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let queries: Option<Vec<QueryType>> = Some(vec![]);
    let status = server
        .post("/admin/db/user/db/exec", &queries, NO_TOKEN)
        .await?
        .0;
    assert_eq!(status, 401);

    Ok(())
}
