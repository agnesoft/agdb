use crate::runner::Test;
use crate::runner::tests;
use crate::test_server::ADMIN;
use crate::test_server::TestServer;
use crate::test_server::next_db_name;
use crate::test_server::next_user_name;
use crate::test_server::test_error::TestError;
use agdb::DbElement;
use agdb::DbId;
use agdb::QueryBuilder;
use agdb::QueryType;
use agdb_api::DbKind;

async fn copy() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    let db2 = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_db_add(owner, db, DbKind::Mapped).await?;
    let queries: &[QueryType] = &[QueryBuilder::insert()
        .nodes()
        .aliases(["root"])
        .query()
        .into()];
    server.api.admin_db_exec_mut(owner, db, queries).await?;
    let status = server.api.admin_db_copy(owner, db, owner, db2).await?;
    assert_eq!(status, 201);
    assert!(
        server
            .api
            .admin_db_list()
            .await?
            .1
            .iter()
            .any(|d| d.owner == *owner && d.db == *db2)
    );
    let queries: &[QueryType] = &[QueryBuilder::select().ids("root").query().into()];
    let results = server.api.admin_db_exec(owner, db2, queries).await?.1;
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].result, 1);
    assert_eq!(
        results[0].elements,
        vec![DbElement {
            id: DbId(1),
            from: DbId::default(),
            to: DbId::default(),
            values: vec![]
        }]
    );
    Ok(())
}

async fn copy_to_different_user() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let owner2 = &next_user_name();
    let db = &next_db_name();
    let db2 = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_user_add(owner2, owner2).await?;
    server.api.admin_db_add(owner, db, DbKind::Mapped).await?;
    let queries: &[QueryType] = &[QueryBuilder::insert()
        .nodes()
        .aliases(["root"])
        .query()
        .into()];
    server.api.admin_db_exec_mut(owner, db, queries).await?;
    let status = server.api.admin_db_copy(owner, db, owner2, db2).await?;
    assert_eq!(status, 201);
    assert!(
        server
            .api
            .admin_db_list()
            .await?
            .1
            .iter()
            .any(|d| d.owner == *owner2 && d.db == *db2)
    );
    assert_eq!(server.api.admin_db_audit(owner2, db2).await?.1.0.len(), 1);
    let queries: &[QueryType] = &[QueryBuilder::select().ids("root").query().into()];
    let results = server.api.admin_db_exec(owner2, db2, queries).await?.1;
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].result, 1);
    assert_eq!(
        results[0].elements,
        vec![DbElement {
            id: DbId(1),
            from: DbId::default(),
            to: DbId::default(),
            values: vec![]
        }]
    );
    server
        .api
        .admin_db_exec_mut(
            owner2,
            db2,
            &[QueryBuilder::remove().ids("root").query().into()],
        )
        .await?;
    assert_eq!(server.api.admin_db_audit(owner2, db2).await?.1.0.len(), 2);
    Ok(())
}

async fn copy_target_exists() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    let db2 = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_db_add(owner, db, DbKind::Memory).await?;
    server.api.admin_db_add(owner, db2, DbKind::Memory).await?;
    let status = server
        .api
        .admin_db_copy(owner, db, owner, db2)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 465);
    Ok(())
}

async fn target_self() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_db_add(owner, db, DbKind::Memory).await?;
    let status = server
        .api
        .admin_db_copy(owner, db, owner, db)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 465);
    Ok(())
}

async fn invalid() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_db_add(owner, db, DbKind::File).await?;
    let status = server
        .api
        .admin_db_copy(owner, db, owner, &format!("{owner}/a\0a"))
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 467);
    Ok(())
}

async fn db_not_found() -> Result<(), TestError> {
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

async fn non_admin() -> Result<(), TestError> {
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

async fn no_token() -> Result<(), TestError> {
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

pub(super) const TESTS: &[Test] = tests![
    copy,
    copy_to_different_user,
    copy_target_exists,
    target_self,
    invalid,
    db_not_found,
    non_admin,
    no_token,
];
