use agdb_api::test_server::test_error::TestError;

#[tokio::test]
async fn change_password() -> Result<(), TestError> {
    agdb_api::tests::routes::user_change_password_test::change_password().await
}

#[tokio::test]
async fn invalid_credentials() -> Result<(), TestError> {
    agdb_api::tests::routes::user_change_password_test::invalid_credentials().await
}

#[tokio::test]
async fn password_too_short() -> Result<(), TestError> {
    agdb_api::tests::routes::user_change_password_test::password_too_short().await
}

#[tokio::test]
async fn no_token() -> Result<(), TestError> {
    agdb_api::tests::routes::user_change_password_test::no_token().await
}
