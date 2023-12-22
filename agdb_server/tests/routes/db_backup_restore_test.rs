use crate::TestServer;
use crate::NO_TOKEN;
use agdb::DbElement;
use agdb::DbId;
use agdb::QueryBuilder;
use agdb::QueryResult;
use agdb::QueryType;
use std::path::Path;

#[tokio::test]
async fn backup() -> anyhow::Result<()> {
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
        .post(&format!("/db/{db}/backup"), &String::new(), &user.token)
        .await?
        .0;
    assert_eq!(status, 201);
    assert!(Path::new(&server.data_dir)
        .join(user.name)
        .join("backups")
        .join(format!("{}.bak", db.split_once('/').unwrap().1))
        .exists());
    let queries: Vec<QueryType> = vec![QueryBuilder::remove().ids("root").query().into()];
    server
        .post(&format!("/db/{db}/exec"), &queries, &user.token)
        .await?;
    let status = server
        .post(&format!("/db/{db}/restore"), &String::new(), &user.token)
        .await?
        .0;
    assert_eq!(status, 201);
    let queries: Vec<QueryType> = vec![QueryBuilder::select().ids("root").query().into()];
    let responses = server
        .post(&format!("/db/{db}/exec"), &queries, &user.token)
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
async fn backup_overwrite() -> anyhow::Result<()> {
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
        .post(&format!("/db/{db}/backup"), &String::new(), &user.token)
        .await?
        .0;
    assert_eq!(status, 201);
    assert!(Path::new(&server.data_dir)
        .join(&user.name)
        .join("backups")
        .join(format!("{}.bak", db.split_once('/').unwrap().1))
        .exists());
    let queries: Vec<QueryType> = vec![QueryBuilder::remove().ids("root").query().into()];
    server
        .post(&format!("/db/{db}/exec"), &queries, &user.token)
        .await?;
    let status = server
        .post(&format!("/db/{db}/backup"), &String::new(), &user.token)
        .await?
        .0;
    assert_eq!(status, 201);
    assert!(Path::new(&server.data_dir)
        .join(user.name)
        .join("backups")
        .join(format!("{}.bak", db.split_once('/').unwrap().1))
        .exists());
    let status = server
        .post(&format!("/db/{db}/restore"), &String::new(), &user.token)
        .await?
        .0;
    assert_eq!(status, 201);
    let queries: Vec<QueryType> = vec![QueryBuilder::select().ids("root").query().into()];
    let responses = server
        .post(&format!("/db/{db}/exec"), &queries, &user.token)
        .await?
        .1;
    assert_eq!(responses, "Alias 'root' not found");

    Ok(())
}

#[tokio::test]
async fn backup_of_backup() -> anyhow::Result<()> {
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
        .post(&format!("/db/{db}/backup"), &String::new(), &user.token)
        .await?
        .0;
    assert_eq!(status, 201);
    assert!(Path::new(&server.data_dir)
        .join(user.name)
        .join("backups")
        .join(format!("{}.bak", db.split_once('/').unwrap().1))
        .exists());
    let queries: Vec<QueryType> = vec![QueryBuilder::remove().ids("root").query().into()];
    server
        .post(&format!("/db/{db}/exec"), &queries, &user.token)
        .await?;
    let status = server
        .post(&format!("/db/{db}/restore"), &String::new(), &user.token)
        .await?
        .0;
    assert_eq!(status, 201);
    let status = server
        .post(&format!("/db/{db}/restore"), &String::new(), &user.token)
        .await?
        .0;
    assert_eq!(status, 201);
    let queries: Vec<QueryType> = vec![QueryBuilder::select().ids("root").query().into()];
    let responses = server
        .post(&format!("/db/{db}/exec"), &queries, &user.token)
        .await?
        .1;
    assert_eq!(responses, "Alias 'root' not found");

    Ok(())
}

#[tokio::test]
async fn restore_no_backup() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let db = server.init_db("memory", &user).await?;
    let status = server
        .post(&format!("/db/{db}/restore"), &String::new(), &user.token)
        .await?
        .0;
    assert_eq!(status, 404);

    Ok(())
}

#[tokio::test]
async fn in_memory() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let db = server.init_db("memory", &user).await?;
    let status = server
        .post(&format!("/db/{db}/backup"), &String::new(), &user.token)
        .await?
        .0;
    assert_eq!(status, 403);

    Ok(())
}

#[tokio::test]
async fn non_admin() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let other = server.init_user().await?;
    let db = server.init_db("mapped", &user).await?;
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
        .post(&format!("/db/{db}/backup"), &String::new(), &other.token)
        .await?
        .0;
    assert_eq!(status, 403);
    let status = server
        .post(&format!("/db/{db}/restore"), &String::new(), &other.token)
        .await?
        .0;
    assert_eq!(status, 403);

    Ok(())
}

#[tokio::test]
async fn no_token() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let status = server
        .post("/db/user/not_found/backup", &String::new(), NO_TOKEN)
        .await?
        .0;
    assert_eq!(status, 401);

    Ok(())
}
