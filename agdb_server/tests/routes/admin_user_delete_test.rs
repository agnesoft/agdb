use agdb_api::test_server::test_error::TestError;

#[tokio::test]
async fn delete() -> Result<(), TestError> {
    agdb_api::tests::routes::admin_user_delete_test::delete().await
}

#[tokio::test]
async fn delete_with_other() -> Result<(), TestError> {
    agdb_api::tests::routes::admin_user_delete_test::delete_with_other().await
}

#[tokio::test]
async fn user_not_found() -> Result<(), TestError> {
    agdb_api::tests::routes::admin_user_delete_test::user_not_found().await
}

#[tokio::test]
async fn non_admin() -> Result<(), TestError> {
    agdb_api::tests::routes::admin_user_delete_test::non_admin().await
}

#[tokio::test]
async fn no_token() -> Result<(), TestError> {
    agdb_api::tests::routes::admin_user_delete_test::no_token().await
}
