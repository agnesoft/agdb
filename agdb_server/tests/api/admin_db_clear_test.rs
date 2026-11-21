use crate::runner::Test;
use crate::runner::tests;
use crate::test_server::ADMIN;
use crate::test_server::TestServer;
use crate::test_server::next_db_name;
use crate::test_server::next_user_name;
use crate::test_server::test_error::TestError;
use agdb::QueryBuilder;
use agdb::QueryType;
use agdb_api::DbKind;
use agdb_api::DbResource;
use agdb_api::DbUserRole;

async fn clear_backup() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.user_login(owner, owner).await?;
    server.api.db_add(owner, db, DbKind::Mapped).await?;
    let queries: &[QueryType] = &[QueryBuilder::insert().nodes().count(1).query().into()];
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

async fn clear_audit() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.user_login(owner, owner).await?;
    server.api.db_add(owner, db, DbKind::Mapped).await?;
    let queries: &[QueryType] = &[QueryBuilder::insert().nodes().count(1).query().into()];
    server.api.db_exec_mut(owner, db, queries).await?;
    server.api.user_login(ADMIN, ADMIN).await?;
    let (status, _) = server
        .api
        .admin_db_clear(owner, db, DbResource::Audit)
        .await?;
    assert_eq!(status, 200);
    assert_eq!(server.api.admin_db_audit(owner, db).await?.1.0.len(), 0);
    Ok(())
}

async fn clear_db() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.user_login(owner, owner).await?;
    server.api.db_add(owner, db, DbKind::Mapped).await?;
    let queries: &[QueryType] = &[QueryBuilder::insert().nodes().count(100).query().into()];
    server.api.db_exec_mut(owner, db, queries).await?;
    let (_, list) = server.api.db_list().await?;
    let original_size = list[0].size;
    server.api.user_login(ADMIN, ADMIN).await?;
    let (status, db) = server.api.admin_db_clear(owner, db, DbResource::Db).await?;
    assert_eq!(status, 200);
    assert!(db.size < original_size);
    Ok(())
}

async fn clear_db_memory() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.user_login(owner, owner).await?;
    server.api.db_add(owner, db, DbKind::Memory).await?;
    let queries: &[QueryType] = &[QueryBuilder::insert().nodes().count(100).query().into()];
    server.api.db_exec_mut(owner, db, queries).await?;
    let (_, list) = server.api.db_list().await?;
    let original_size = list[0].size;
    server.api.user_login(ADMIN, ADMIN).await?;
    let (status, db) = server.api.admin_db_clear(owner, db, DbResource::Db).await?;
    assert_eq!(status, 200);
    assert!(db.size < original_size);
    Ok(())
}

async fn clear_all() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.user_login(owner, owner).await?;
    server.api.db_add(owner, db, DbKind::Mapped).await?;
    let queries: &[QueryType] = &[QueryBuilder::insert().nodes().count(100).query().into()];
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
    assert_eq!(server.api.admin_db_audit(owner, db).await?.1.0.len(), 0);
    assert_eq!(database.backup, 0);
    Ok(())
}

async fn non_admin() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let user = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_user_add(user, user).await?;
    server.api.admin_db_add(owner, db, DbKind::Memory).await?;
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

async fn no_token() -> Result<(), TestError> {
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

pub(super) const TESTS: &[Test] = tests![
    clear_backup,
    clear_audit,
    clear_db,
    clear_db_memory,
    clear_all,
    non_admin,
    no_token,
];
