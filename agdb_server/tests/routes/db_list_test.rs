use agdb_api::test_server::test_error::TestError;

#[tokio::test]
async fn list() -> Result<(), TestError> {
    agdb_api::tests::routes::db_list_test::list().await
}

#[tokio::test]
async fn with_backup() -> Result<(), TestError> {
    agdb_api::tests::routes::db_list_test::with_backup().await
}

#[tokio::test]
async fn list_empty() -> Result<(), TestError> {
    agdb_api::tests::routes::db_list_test::list_empty().await
}

#[tokio::test]
async fn list_no_token() -> Result<(), TestError> {
    agdb_api::tests::routes::db_list_test::list_no_token().await
}
