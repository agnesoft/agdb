use agdb_api::test_server::test_error::TestError;

#[tokio::test]
async fn audit_trim() -> Result<(), TestError> {
    agdb_api::tests::routes::db_audit_trim_test::audit_trim().await
}

#[tokio::test]
async fn audit_trim_non_admin() -> Result<(), TestError> {
    agdb_api::tests::routes::db_audit_trim_test::audit_trim_non_admin().await
}

#[tokio::test]
async fn audit_trim_db_not_found() -> Result<(), TestError> {
    agdb_api::tests::routes::db_audit_trim_test::audit_trim_db_not_found().await
}

#[tokio::test]
async fn audit_trim_no_token() -> Result<(), TestError> {
    agdb_api::tests::routes::db_audit_trim_test::audit_trim_no_token().await
}
