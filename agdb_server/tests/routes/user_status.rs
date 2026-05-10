use agdb_api::test_server::test_error::TestError;

#[tokio::test]
async fn user() -> Result<(), TestError> {
    agdb_api::tests::routes::user_status::user().await
}

#[tokio::test]
async fn admin() -> Result<(), TestError> {
    agdb_api::tests::routes::user_status::admin().await
}

#[tokio::test]
async fn no_token() -> Result<(), TestError> {
    agdb_api::tests::routes::user_status::no_token().await
}
