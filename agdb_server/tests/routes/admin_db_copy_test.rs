use crate::ADMIN;
use crate::TestCluster;
use crate::TestServer;
use crate::next_db_name;
use crate::next_user_name;
use agdb::DbElement;
use agdb::DbId;
use agdb::QueryBuilder;
use agdb_api::DbType;
use std::path::Path;

#[tokio::test]
async fn copy() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    let db2 = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_db_add(owner, db, DbType::Mapped).await?;
    let queries = &vec![
        QueryBuilder::insert()
            .nodes()
            .aliases(["root"])
            .query()
            .into(),
    ];
    server.api.admin_db_exec_mut(owner, db, queries).await?;
    let status = server.api.admin_db_copy(owner, db, owner, db2).await?;
    assert_eq!(status, 201);
    assert!(Path::new(&server.data_dir).join(owner).join(db2).exists());
    let queries = &vec![QueryBuilder::select().ids("root").query().into()];
    let results = server.api.admin_db_exec(owner, db2, queries).await?.1;
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].result, 1);
    assert_eq!(
        results[0].elements,
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
async fn copy_to_different_user() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let owner2 = &next_user_name();
    let db = &next_db_name();
    let db2 = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_user_add(owner2, owner2).await?;
    server.api.admin_db_add(owner, db, DbType::Mapped).await?;
    let queries = &vec![
        QueryBuilder::insert()
            .nodes()
            .aliases(["root"])
            .query()
            .into(),
    ];
    server.api.admin_db_exec_mut(owner, db, queries).await?;
    let status = server.api.admin_db_copy(owner, db, owner2, db2).await?;
    assert_eq!(status, 201);
    assert!(Path::new(&server.data_dir).join(owner2).join(db2).exists());
    let queries = &vec![QueryBuilder::select().ids("root").query().into()];
    let results = server.api.admin_db_exec(owner2, db2, queries).await?.1;
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].result, 1);
    assert_eq!(
        results[0].elements,
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
async fn copy_target_exists() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    let db2 = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_db_add(owner, db, DbType::Memory).await?;
    server.api.admin_db_add(owner, db2, DbType::Memory).await?;
    let status = server
        .api
        .admin_db_copy(owner, db, owner, db2)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 465);
    Ok(())
}

#[tokio::test]
async fn target_self() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_db_add(owner, db, DbType::Memory).await?;
    let status = server
        .api
        .admin_db_copy(owner, db, owner, db)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 465);
    Ok(())
}

#[tokio::test]
async fn invalid() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_db_add(owner, db, DbType::File).await?;
    let status = server
        .api
        .admin_db_copy(owner, db, owner, &format!("{owner}/a\0a"))
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 467);
    Ok(())
}

#[tokio::test]
async fn db_not_found() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    let status = server
        .api
        .admin_db_copy(owner, "db", owner, "dbx")
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
    let db = &next_db_name();
    let db2 = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.user_login(owner, owner).await?;
    let status = server
        .api
        .admin_db_copy(owner, db, owner, db2)
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
        .admin_db_copy("owner", "db", "owner", "dbx")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 401);
    Ok(())
}

#[tokio::test]
async fn cluster_copy() -> anyhow::Result<()> {
    let mut cluster = TestCluster::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    let db2 = &next_db_name();
    let client = cluster.apis.get_mut(1).unwrap();
    client.user_login(ADMIN, ADMIN).await?;
    client.admin_user_add(owner, owner).await?;
    client.admin_db_add(owner, db, DbType::Memory).await?;
    client.admin_db_copy(owner, db, owner, db2).await?;
    client.user_login(owner, owner).await?;
    let dbs = client.db_list().await?.1.len();
    assert_eq!(dbs, 2);
    Ok(())
}
