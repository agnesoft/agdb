use crate::next_db_name;
use crate::next_user_name;
use crate::TestCluster;
use crate::TestServer;
use crate::ADMIN;
use agdb::QueryBuilder;
use agdb_api::DbResource;
use agdb_api::DbType;
use agdb_api::DbUserRole;
use std::path::Path;

#[tokio::test]
async fn clear_backup() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.user_login(owner, owner).await?;
    server.api.db_add(owner, db, DbType::Mapped).await?;
    let queries = &vec![QueryBuilder::insert().nodes().count(1).query().into()];
    server.api.db_exec(owner, db, queries).await?;
    server.api.db_backup(owner, db).await?;
    let (status, db) = server.api.db_clear(owner, db, DbResource::Backup).await?;
    assert_eq!(status, 200);
    assert_eq!(db.backup, 0);
    Ok(())
}

#[tokio::test]
async fn clear_audit() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.user_login(owner, owner).await?;
    server.api.db_add(owner, db, DbType::Mapped).await?;
    let queries = &vec![QueryBuilder::insert().nodes().count(1).query().into()];
    server.api.db_exec(owner, db, queries).await?;
    let (status, _) = server.api.db_clear(owner, db, DbResource::Audit).await?;
    assert_eq!(status, 200);
    let db_audit_file = Path::new(&server.data_dir)
        .join(owner)
        .join("audit")
        .join(format!("{}.log", db));
    assert!(!db_audit_file.exists());
    Ok(())
}

#[tokio::test]
async fn clear_db() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.user_login(owner, owner).await?;
    server.api.db_add(owner, db, DbType::Mapped).await?;
    let queries = &vec![QueryBuilder::insert().nodes().count(100).query().into()];
    server.api.db_exec(owner, db, queries).await?;
    let (_, list) = server.api.db_list().await?;
    let original_size = list[0].size;
    let (status, db) = server.api.db_clear(owner, db, DbResource::Db).await?;
    assert_eq!(status, 200);
    assert!(db.size < original_size);
    Ok(())
}

#[tokio::test]
async fn clear_db_memory() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.user_login(owner, owner).await?;
    server.api.db_add(owner, db, DbType::Memory).await?;
    let queries = &vec![QueryBuilder::insert().nodes().count(100).query().into()];
    server.api.db_exec(owner, db, queries).await?;
    let (_, list) = server.api.db_list().await?;
    let original_size = list[0].size;
    let (status, db) = server.api.db_clear(owner, db, DbResource::Db).await?;
    assert_eq!(status, 200);
    assert!(db.size < original_size);
    Ok(())
}

#[tokio::test]
async fn clear_db_memory_backup() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.user_login(owner, owner).await?;
    server.api.db_add(owner, db, DbType::Memory).await?;
    let db_path = Path::new(&server.data_dir).join(owner).join(db);
    assert!(!db_path.exists());
    server.api.db_backup(owner, db).await?;
    assert!(db_path.exists());
    let (status, db) = server.api.db_clear(owner, db, DbResource::Backup).await?;
    assert!(!db_path.exists());
    assert_eq!(status, 200);
    assert_eq!(db.backup, 0);
    Ok(())
}

#[tokio::test]
async fn clear_all() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.user_login(owner, owner).await?;
    server.api.db_add(owner, db, DbType::Mapped).await?;
    let queries = &vec![QueryBuilder::insert().nodes().count(100).query().into()];
    server.api.db_exec(owner, db, queries).await?;
    let (_, list) = server.api.db_list().await?;
    let original_size = list[0].size;
    server.api.db_backup(owner, db).await?;
    let (status, database) = server.api.db_clear(owner, db, DbResource::All).await?;
    assert_eq!(status, 200);
    assert!(database.size < original_size);
    let db_audit_file = Path::new(&server.data_dir)
        .join(owner)
        .join("audit")
        .join(format!("{}.log", db));
    assert!(!db_audit_file.exists());
    assert_eq!(database.backup, 0);
    Ok(())
}

#[tokio::test]
async fn non_admin() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let user = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_user_add(user, user).await?;
    server.api.admin_db_add(owner, db, DbType::Memory).await?;
    server
        .api
        .admin_db_user_add(owner, db, user, DbUserRole::Write)
        .await?;
    server.api.user_login(user, user).await?;
    let status = server
        .api
        .db_clear(owner, db, DbResource::All)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 403);
    Ok(())
}

#[tokio::test]
async fn no_token() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let status = server
        .api
        .db_clear("owner", "db", DbResource::All)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 401);
    Ok(())
}

#[tokio::test]
async fn cluster_clear() -> anyhow::Result<()> {
    let mut cluster = TestCluster::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    let client = cluster.apis.get_mut(1).unwrap();
    client.cluster_login(ADMIN, ADMIN).await?;
    client.admin_user_add(owner, owner).await?;
    client.cluster_login(owner, owner).await?;
    client.db_add(owner, db, DbType::Memory).await?;
    client
        .db_exec_mut(
            owner,
            db,
            &[QueryBuilder::insert().nodes().count(1).query().into()],
        )
        .await?;
    let node_count_query = &[QueryBuilder::select().node_count().query().into()];
    let node_count = client.db_exec(owner, db, node_count_query).await?.1[0].elements[0].values[0]
        .value
        .to_u64()?;
    assert_eq!(node_count, 1);
    client.db_clear(owner, db, DbResource::All).await?;
    let node_count = client.db_exec(owner, db, node_count_query).await?.1[0].elements[0].values[0]
        .value
        .to_u64()?;
    assert_eq!(node_count, 0);
    Ok(())
}
