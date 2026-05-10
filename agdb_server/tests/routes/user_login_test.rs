use agdb_api::test_server::test_error::TestError;

#[tokio::test]
async fn login() -> Result<(), TestError> {
    agdb_api::tests::routes::user_login_test::login().await
}

#[tokio::test]
async fn repeated_login() -> Result<(), TestError> {
    agdb_api::tests::routes::user_login_test::repeated_login().await
}

#[tokio::test]
async fn invalid_credentials() -> Result<(), TestError> {
    agdb_api::tests::routes::user_login_test::invalid_credentials().await
}

#[tokio::test]
async fn user_not_found() -> Result<(), TestError> {
    agdb_api::tests::routes::user_login_test::user_not_found().await
}

#[tokio::test]
async fn concurrent_logins() -> Result<(), TestError> {
    agdb_api::tests::routes::user_login_test::concurrent_logins().await
}

