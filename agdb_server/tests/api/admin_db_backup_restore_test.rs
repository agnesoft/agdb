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

async fn backup_overwrite() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_db_add(owner, db, DbKind::Mapped).await?;
    let queries: &[QueryType] = &[QueryBuilder::insert()
        .nodes()
        .aliases(["root"])
        .query()
        .into()];
    server.api.admin_db_exec_mut(owner, db, queries).await?;
    let status = server.api.admin_db_backup(owner, db).await?;
    assert_eq!(status, 201);
    assert!(
        server
            .api
            .admin_db_list()
            .await?
            .1
            .iter()
            .any(|d| d.owner == *owner && d.db == *db && d.backup != 0)
    );
    let queries: &[QueryType] = &[QueryBuilder::remove().ids("root").query().into()];
    server.api.admin_db_exec_mut(owner, db, queries).await?;
    let status = server.api.admin_db_backup(owner, db).await?;
    assert_eq!(status, 201);
    assert!(
        server
            .api
            .admin_db_list()
            .await?
            .1
            .iter()
            .any(|d| d.owner == *owner && d.db == *db && d.backup != 0)
    );
    let status = server.api.admin_db_restore(owner, db).await?;
    assert_eq!(status, 201);
    let queries: &[QueryType] = &[QueryBuilder::select().ids("root").query().into()];
    let results = server
        .api
        .admin_db_exec(owner, db, queries)
        .await
        .unwrap_err()
        .description;
    assert_eq!(results, "Alias 'root' not found");
    Ok(())
}

async fn backup_of_backup() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_db_add(owner, db, DbKind::Mapped).await?;
    let queries: &[QueryType] = &[QueryBuilder::insert()
        .nodes()
        .aliases(["root"])
        .query()
        .into()];
    server.api.admin_db_exec_mut(owner, db, queries).await?;
    let status = server.api.admin_db_backup(owner, db).await?;
    assert_eq!(status, 201);
    assert!(
        server
            .api
            .admin_db_list()
            .await?
            .1
            .iter()
            .any(|d| d.owner == *owner && d.db == *db && d.backup != 0)
    );
    let queries: &[QueryType] = &[QueryBuilder::remove().ids("root").query().into()];
    server.api.admin_db_exec_mut(owner, db, queries).await?;
    let status = server.api.admin_db_restore(owner, db).await?;
    assert_eq!(status, 201);
    let status = server.api.admin_db_restore(owner, db).await?;
    assert_eq!(status, 201);
    let queries: &[QueryType] = &[QueryBuilder::select().ids("root").query().into()];
    let results = server.api.admin_db_exec(owner, db, queries).await?.1;
    assert_eq!(results[0].result, 1);

    Ok(())
}

async fn in_memory() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_db_add(owner, db, DbKind::Memory).await?;
    let status = server.api.admin_db_backup(owner, db).await?;
    assert_eq!(status, 201);
    assert!(
        server
            .api
            .admin_db_list()
            .await?
            .1
            .iter()
            .any(|d| d.owner == *owner && d.db == *db && d.backup != 0)
    );
    let status = server.api.admin_db_restore(owner, db).await?;
    assert_eq!(status, 201);
    let status = server.api.admin_db_rollback(owner, db).await?;
    assert_eq!(status, 201);
    Ok(())
}

async fn restore_no_backup() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_db_add(owner, db, DbKind::Mapped).await?;
    let status = server
        .api
        .admin_db_restore(owner, db)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 404);
    let status = server
        .api
        .admin_db_rollback(owner, db)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 404);
    Ok(())
}

async fn non_admin() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_db_add(owner, db, DbKind::Memory).await?;
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
    let status = server
        .api
        .admin_db_rollback(owner, db)
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
    let status = server
        .api
        .admin_db_rollback("owner", "db")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 401);
    Ok(())
}

pub(super) const TESTS: &[Test] = tests![
    backup_overwrite,
    backup_of_backup,
    in_memory,
    restore_no_backup,
    non_admin,
    no_token,
];
