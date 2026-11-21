use crate::runner::Test;
use crate::runner::tests;
use crate::test_server::ADMIN;
use crate::test_server::TestServer;
use crate::test_server::next_db_name;
use crate::test_server::next_user_name;
use crate::test_server::test_error::TestError;
use agdb_api::DbKind;
use agdb_api::DbUserRole;

const KEEP_DAYS: u64 = 1;

async fn audit_trim_non_admin() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let user = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_user_add(user, user).await?;
    server.api.user_login(owner, owner).await?;
    server.api.db_add(owner, db, DbKind::Mapped).await?;
    server
        .api
        .db_user_add(owner, db, user, DbUserRole::Write)
        .await?;
    server.api.user_login(user, user).await?;
    let status = server
        .api
        .db_audit_trim(owner, db, KEEP_DAYS)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 403);
    Ok(())
}

async fn audit_trim_db_not_found() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.user_login(owner, owner).await?;
    let status = server
        .api
        .db_audit_trim(owner, "missing_db", KEEP_DAYS)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 404);
    Ok(())
}

async fn audit_trim_no_token() -> Result<(), TestError> {
    let server = TestServer::new().await?;
    let status = server
        .api
        .db_audit_trim("owner", "db", KEEP_DAYS)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 401);
    Ok(())
}

pub(super) const TESTS: &[Test] = tests![
    audit_trim_non_admin,
    audit_trim_db_not_found,
    audit_trim_no_token,
];
