use crate::TestServer;
use agdb::DbElement;
use agdb::DbId;
use agdb::QueryBuilder;
use agdb::QueryResult;
use agdb::QueryType;

#[tokio::test]
async fn exec() -> anyhow::Result<()> {
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
