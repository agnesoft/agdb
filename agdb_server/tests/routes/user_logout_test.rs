use agdb_api::test_server::test_error::TestError;

#[tokio::test]
async fn logout() -> Result<(), TestError> {
    agdb_api::tests::routes::user_logout_test::logout().await
}

#[tokio::test]
async fn no_token() -> Result<(), TestError> {
    agdb_api::tests::routes::user_logout_test::no_token().await
}

