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
use std::path::Path;

#[tokio::test]
async fn backup() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_db_add(owner, db, DbType::Mapped).await?;
    let queries = &vec![QueryBuilder::insert()
        .nodes()
        .aliases(["root"])
        .query()
        .into()];
    server.api.admin_db_exec_mut(owner, db, queries).await?;
    let status = server.api.admin_db_backup(owner, db).await?;
    assert_eq!(status, 201);
    assert!(Path::new(&server.data_dir)
        .join(owner)
        .join("backups")
        .join(format!("{}.bak", db))
        .exists());
    let queries = &vec![QueryBuilder::remove().ids("root").query().into()];
    server.api.admin_db_exec_mut(owner, db, queries).await?;
    let status = server.api.admin_db_restore(owner, db).await?;
    assert_eq!(status, 201);
    let queries = &vec![QueryBuilder::select().ids("root").query().into()];
    let results = server.api.admin_db_exec(owner, db, queries).await?.1;
    assert_eq!(
        results,
        vec![QueryResult {
            result: 1,
            elements: vec![DbElement {
                id: DbId(1),
                from: None,
                to: None,
                values: vec![]
            }]
        }]
    );
    Ok(())
}

#[tokio::test]
async fn backup_overwrite() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_db_add(owner, db, DbType::Mapped).await?;
    let queries = &vec![QueryBuilder::insert()
        .nodes()
        .aliases(["root"])
        .query()
        .into()];
    server.api.admin_db_exec_mut(owner, db, queries).await?;
    let status = server.api.admin_db_backup(owner, db).await?;
    assert_eq!(status, 201);
    assert!(Path::new(&server.data_dir)
        .join(owner)
        .join("backups")
        .join(format!("{}.bak", db))
        .exists());
    let queries = &vec![QueryBuilder::remove().ids("root").query().into()];
    server.api.admin_db_exec_mut(owner, db, queries).await?;
    let status = server.api.admin_db_backup(owner, db).await?;
    assert_eq!(status, 201);
    assert!(Path::new(&server.data_dir)
        .join(owner)
        .join("backups")
        .join(format!("{}.bak", db))
        .exists());
    let status = server.api.admin_db_restore(owner, db).await?;
    assert_eq!(status, 201);
    let queries = &vec![QueryBuilder::select().ids("root").query().into()];
    let results = server
        .api
        .admin_db_exec(owner, db, queries)
        .await
        .unwrap_err()
        .description;
    assert_eq!(results, "Alias 'root' not found");
    Ok(())
}

#[tokio::test]
async fn backup_of_backup() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_db_add(owner, db, DbType::Mapped).await?;
    let queries = &vec![QueryBuilder::insert()
        .nodes()
        .aliases(["root"])
        .query()
        .into()];
    server.api.admin_db_exec_mut(owner, db, queries).await?;
    let status = server.api.admin_db_backup(owner, db).await?;
    assert_eq!(status, 201);
    assert!(Path::new(&server.data_dir)
        .join(owner)
        .join("backups")
        .join(format!("{}.bak", db))
        .exists());
    let queries = &vec![QueryBuilder::remove().ids("root").query().into()];
    server.api.admin_db_exec_mut(owner, db, queries).await?;
    let status = server.api.admin_db_restore(owner, db).await?;
    assert_eq!(status, 201);
    let status = server.api.admin_db_restore(owner, db).await?;
    assert_eq!(status, 201);
    let queries = &vec![QueryBuilder::select().ids("root").query().into()];
    let results = server
        .api
        .admin_db_exec(owner, db, queries)
        .await
        .unwrap_err()
        .description;
    assert_eq!(results, "Alias 'root' not found");

    Ok(())
}

#[tokio::test]
async fn in_memory() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_db_add(owner, db, DbType::Memory).await?;
    let status = server.api.admin_db_backup(owner, db).await?;
    assert_eq!(status, 201);
    assert!(Path::new(&server.data_dir).join(owner).join(db).exists());
    Ok(())
}

#[tokio::test]
async fn restore_no_backup() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_db_add(owner, db, DbType::Mapped).await?;
    let status = server
        .api
        .admin_db_restore(owner, db)
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
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_db_add(owner, db, DbType::Memory).await?;
    server.api.user_login(owner, owner).await?;
    let status = server
        .api
        .admin_db_backup(owner, db)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 401);
    let status = server
        .api
        .admin_db_restore(owner, db)
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
        .admin_db_backup("owner", "db")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 401);
    let status = server
        .api
        .admin_db_restore("owner", "db")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 401);
    Ok(())
}

#[tokio::test]
async fn cluster_backup() -> anyhow::Result<()> {
    let mut cluster = TestCluster::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    let client = cluster.apis.get_mut(1).unwrap();
    client.cluster_login(ADMIN, ADMIN).await?;
    client.admin_user_add(owner, owner).await?;
    client.admin_db_add(owner, db, DbType::Memory).await?;
    client.admin_db_backup(owner, db).await?;
    client
        .admin_db_exec_mut(
            owner,
            db,
            &[QueryBuilder::insert().nodes().count(1).query().into()],
        )
        .await?;
    let node_count_query = &[QueryBuilder::select().node_count().query().into()];
    let node_count = client.admin_db_exec(owner, db, node_count_query).await?.1[0].elements[0]
        .values[0]
        .value
        .to_u64()?;
    assert_eq!(node_count, 1);
    client.admin_db_restore(owner, db).await?;
    let node_count = client.admin_db_exec(owner, db, node_count_query).await?.1[0].elements[0]
        .values[0]
        .value
        .to_u64()?;
    assert_eq!(node_count, 0);
    Ok(())
}
