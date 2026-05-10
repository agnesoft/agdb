use agdb_api::test_server::test_error::TestError;

#[tokio::test]
async fn add() -> Result<(), TestError> {
    agdb_api::tests::routes::db_add_test::add().await
}

#[tokio::test]
async fn add_same_name_with_previous_backup_after_delete() -> Result<(), TestError> {
    agdb_api::tests::routes::db_add_test::add_same_name_with_previous_backup_after_delete().await
}

#[tokio::test]
async fn add_same_name_with_backup_after_remove() -> Result<(), TestError> {
    agdb_api::tests::routes::db_add_test::add_same_name_with_backup_after_remove().await
}

#[tokio::test]
async fn add_same_name_different_user() -> Result<(), TestError> {
    agdb_api::tests::routes::db_add_test::add_same_name_different_user().await
}

#[tokio::test]
async fn db_already_exists() -> Result<(), TestError> {
    agdb_api::tests::routes::db_add_test::db_already_exists().await
}

#[tokio::test]
async fn db_user_mismatch() -> Result<(), TestError> {
    agdb_api::tests::routes::db_add_test::db_user_mismatch().await
}

#[tokio::test]
async fn add_db_other_user() -> Result<(), TestError> {
    agdb_api::tests::routes::db_add_test::add_db_other_user().await
}

#[tokio::test]
async fn db_type_invalid() -> Result<(), TestError> {
    agdb_api::tests::routes::db_add_test::db_type_invalid().await
}

#[tokio::test]
async fn no_token() -> Result<(), TestError> {
    agdb_api::tests::routes::db_add_test::no_token().await
}

