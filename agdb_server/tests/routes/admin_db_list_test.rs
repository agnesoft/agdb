use agdb_api::test_server::test_error::TestError;

#[tokio::test]
async fn db_list() -> Result<(), TestError> {
    agdb_api::tests::routes::admin_db_list_test::db_list().await
}

#[tokio::test]
async fn with_backup() -> Result<(), TestError> {
    agdb_api::tests::routes::admin_db_list_test::with_backup().await
}

#[tokio::test]
async fn non_admin() -> Result<(), TestError> {
    agdb_api::tests::routes::admin_db_list_test::non_admin().await
}

#[tokio::test]
async fn no_admin_token() -> Result<(), TestError> {
    agdb_api::tests::routes::admin_db_list_test::no_admin_token().await
}
