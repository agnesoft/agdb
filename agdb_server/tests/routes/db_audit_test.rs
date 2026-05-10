use agdb_api::test_server::test_error::TestError;

#[tokio::test]
async fn audit() -> Result<(), TestError> {
    agdb_api::tests::routes::db_audit_test::audit().await
}

#[tokio::test]
async fn audit_delete_db() -> Result<(), TestError> {
    agdb_api::tests::routes::db_audit_test::audit_delete_db().await
}

#[tokio::test]
async fn audit_db_empty() -> Result<(), TestError> {
    agdb_api::tests::routes::db_audit_test::audit_db_empty().await
}

#[tokio::test]
async fn audit_no_token() -> Result<(), TestError> {
    agdb_api::tests::routes::db_audit_test::audit_no_token().await
}

#[tokio::test]
async fn repeated_query_with_db_audit() -> Result<(), TestError> {
    agdb_api::tests::routes::db_audit_test::repeated_query_with_db_audit().await
}
