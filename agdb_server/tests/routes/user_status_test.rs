use agdb_api::test_server::test_error::TestError;

#[tokio::test]
async fn user() -> Result<(), TestError> {
    agdb_api::tests::routes::user_status_test::user().await
}

#[tokio::test]
async fn admin() -> Result<(), TestError> {
    agdb_api::tests::routes::user_status_test::admin().await
}

#[tokio::test]
async fn no_token() -> Result<(), TestError> {
    agdb_api::tests::routes::user_status_test::no_token().await
}

#[tokio::test]
async fn custom_agent() -> Result<(), TestError> {
    agdb_api::tests::routes::user_status_test::custom_agent().await
}
