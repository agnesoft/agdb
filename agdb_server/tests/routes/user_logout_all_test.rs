use agdb_api::test_server::test_error::TestError;

#[tokio::test]
async fn logout_all() -> Result<(), TestError> {
    agdb_api::tests::routes::user_logout_all_test::logout_all().await
}

#[tokio::test]
async fn no_token() -> Result<(), TestError> {
    agdb_api::tests::routes::user_logout_all_test::no_token().await
}
