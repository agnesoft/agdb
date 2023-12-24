use crate::TestServer;
use crate::NO_TOKEN;
use agdb::DbElement;
use agdb::DbId;
use agdb::QueryBuilder;
use agdb::QueryResult;
use agdb::QueryType;
use std::path::Path;

#[tokio::test]
async fn copy() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let db = server.init_db("mapped", &user).await?;
    let queries: Vec<QueryType> = vec![QueryBuilder::insert()
        .nodes()
        .aliases(vec!["root"])
        .query()
        .into()];
    server
        .post(&format!("/db/{db}/exec"), &queries, &user.token)
        .await?;
    let status = server
        .post(
            &format!("/db/{db}/copy?new_name={}/new_name", user.name),
            &String::new(),
            &user.token,
        )
        .await?
        .0;
    assert_eq!(status, 201);
    assert!(Path::new(&server.data_dir)
        .join(&user.name)
        .join("new_name")
        .exists());
    let queries: Vec<QueryType> = vec![QueryBuilder::select().ids("root").query().into()];
    let responses = server
        .post(
            &format!("/db/{}/new_name/exec", user.name),
            &queries,
            &user.token,
        )
        .await?
        .1;
    let responses: Vec<QueryResult> = serde_json::from_str(&responses)?;
    assert_eq!(responses.len(), 1);
    assert_eq!(responses[0].result, 1);
    assert_eq!(
        responses[0].elements,
        vec![DbElement {
            id: DbId(1),
            from: None,
            to: None,
            values: vec![]
        }]
    );

    Ok(())
}

#[tokio::test]
async fn copy_other() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let other = server.init_user().await?;
    let db = server.init_db("mapped", &user).await?;
    let queries: Vec<QueryType> = vec![QueryBuilder::insert()
        .nodes()
        .aliases(vec!["root"])
        .query()
        .into()];
    server
        .post(&format!("/db/{db}/exec"), &queries, &other.token)
        .await?;
    assert_eq!(
        server
            .put(
                &format!("/db/{db}/user/{}/add?db_role=write", other.name),
                &String::new(),
                &user.token
            )
            .await?,
        201
    );
    let status = server
        .post(
            &format!("/db/{db}/copy?new_name={}/new_name", other.name),
            &String::new(),
            &user.token,
        )
        .await?
        .0;
    assert_eq!(status, 201);
    assert!(Path::new(&server.data_dir)
        .join(&other.name)
        .join("new_name")
        .exists());
    let queries: Vec<QueryType> = vec![QueryBuilder::select().ids("root").query().into()];
    let responses = server
        .post(
            &format!("/db/{}/new_name/exec", other.name),
            &queries,
            &other.token,
        )
        .await?
        .1;
    let responses: Vec<QueryResult> = serde_json::from_str(&responses)?;
    assert_eq!(responses.len(), 1);
    assert_eq!(responses[0].result, 1);
    assert_eq!(
        responses[0].elements,
        vec![DbElement {
            id: DbId(1),
            from: None,
            to: None,
            values: vec![]
        }]
    );

    Ok(())
}

#[tokio::test]
async fn copy_to_other_user() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let db = server.init_db("mapped", &user).await?;
    let status = server
        .post(
            &format!("/db/{db}/copy?new_name=other/new_name"),
            &String::new(),
            &user.token,
        )
        .await?
        .0;
    assert_eq!(status, 403);

    Ok(())
}

#[tokio::test]
async fn copy_target_exists() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let db = server.init_db("mapped", &user).await?;
    let status = server
        .post(
            &format!("/db/{db}/copy?new_name={db}"),
            &String::new(),
            &user.token,
        )
        .await?
        .0;
    assert_eq!(status, 403);

    Ok(())
}

#[tokio::test]
async fn invalid() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let db = server.init_db("mapped", &user).await?;
    let status = server
        .post(
            &format!("/db/{db}/copy?new_name={}/a\0a", user.name),
            &String::new(),
            &user.token,
        )
        .await?
        .0;
    assert_eq!(status, 467);
    Ok(())
}

#[tokio::test]
async fn not_found() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let status = server
        .post(
            &format!("/db/{}/not_found/copy?new_name=user/not_found", user.name),
            &String::new(),
            &user.token,
        )
        .await?
        .0;
    assert_eq!(status, 404);

    Ok(())
}

#[tokio::test]
async fn no_token() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let status = server
        .post(
            "/db/user/not_found/copy?new_name=user/not_found",
            &String::new(),
            NO_TOKEN,
        )
        .await?
        .0;
    assert_eq!(status, 401);

    Ok(())
}
