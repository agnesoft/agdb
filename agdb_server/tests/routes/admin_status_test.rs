use agdb_api::test_server::test_error::TestError;

#[tokio::test]
async fn status() -> Result<(), TestError> {
    agdb_api::tests::routes::admin_status_test::status().await
}

#[tokio::test]
async fn non_admin() -> Result<(), TestError> {
    agdb_api::tests::routes::admin_status_test::non_admin().await
}

#[tokio::test]
async fn no_token() -> Result<(), TestError> {
    agdb_api::tests::routes::admin_status_test::no_token().await
}
