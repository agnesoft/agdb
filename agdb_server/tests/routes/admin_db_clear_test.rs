use crate::ADMIN;
use crate::TestServer;
use crate::next_db_name;
use crate::next_user_name;
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
    server.api.db_exec_mut(owner, db, queries).await?;
    server.api.db_backup(owner, db).await?;
    server.api.user_login(ADMIN, ADMIN).await?;
    let (status, db) = server
        .api
        .admin_db_clear(owner, db, DbResource::Backup)
        .await?;
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
    server.api.db_exec_mut(owner, db, queries).await?;
    server.api.user_login(ADMIN, ADMIN).await?;
    let (status, _) = server
        .api
        .admin_db_clear(owner, db, DbResource::Audit)
        .await?;
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
    server.api.db_exec_mut(owner, db, queries).await?;
    let (_, list) = server.api.db_list().await?;
    let original_size = list[0].size;
    server.api.user_login(ADMIN, ADMIN).await?;
    let (status, db) = server.api.admin_db_clear(owner, db, DbResource::Db).await?;
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
    server.api.db_exec_mut(owner, db, queries).await?;
    let (_, list) = server.api.db_list().await?;
    let original_size = list[0].size;
    server.api.user_login(ADMIN, ADMIN).await?;
    let (status, db) = server.api.admin_db_clear(owner, db, DbResource::Db).await?;
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
    server.api.user_login(ADMIN, ADMIN).await?;
    let (status, db) = server
        .api
        .admin_db_clear(owner, db, DbResource::Backup)
        .await?;
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
    server.api.db_exec_mut(owner, db, queries).await?;
    let (_, list) = server.api.db_list().await?;
    let original_size = list[0].size;
    server.api.db_backup(owner, db).await?;
    server.api.user_login(ADMIN, ADMIN).await?;
    let (status, database) = server
        .api
        .admin_db_clear(owner, db, DbResource::All)
        .await?;
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
        .admin_db_clear(owner, db, DbResource::All)
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
        .admin_db_clear("owner", "db", DbResource::All)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 401);
    Ok(())
}
