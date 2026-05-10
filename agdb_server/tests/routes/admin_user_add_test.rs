use agdb_api::test_server::test_error::TestError;

#[tokio::test]
async fn add() -> Result<(), TestError> {
    agdb_api::tests::routes::admin_user_add_test::add().await
}

#[tokio::test]
async fn add_existing() -> Result<(), TestError> {
    agdb_api::tests::routes::admin_user_add_test::add_existing().await
}

#[tokio::test]
async fn name_too_short() -> Result<(), TestError> {
    agdb_api::tests::routes::admin_user_add_test::name_too_short().await
}

#[tokio::test]
async fn password_too_short() -> Result<(), TestError> {
    agdb_api::tests::routes::admin_user_add_test::password_too_short().await
}

#[tokio::test]
async fn non_admin() -> Result<(), TestError> {
    agdb_api::tests::routes::admin_user_add_test::non_admin().await
}

#[tokio::test]
async fn no_token() -> Result<(), TestError> {
    agdb_api::tests::routes::admin_user_add_test::no_token().await
}

