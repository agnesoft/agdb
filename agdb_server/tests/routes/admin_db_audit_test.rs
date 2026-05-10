use agdb_api::test_server::test_error::TestError;

#[tokio::test]
async fn admin_audit() -> Result<(), TestError> {
    agdb_api::tests::routes::admin_db_audit_test::admin_audit().await
}

#[tokio::test]
async fn admin_audit_db_empty() -> Result<(), TestError> {
    agdb_api::tests::routes::admin_db_audit_test::admin_audit_db_empty().await
}

#[tokio::test]
async fn non_admin() -> Result<(), TestError> {
    agdb_api::tests::routes::admin_db_audit_test::non_admin().await
}

#[tokio::test]
async fn audit_no_token() -> Result<(), TestError> {
    agdb_api::tests::routes::admin_db_audit_test::audit_no_token().await
}

