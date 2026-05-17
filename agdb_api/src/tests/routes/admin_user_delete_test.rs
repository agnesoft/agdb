use crate::DbKind;
use crate::DbUserRole;
use crate::test_server::ADMIN;
use crate::test_server::TestServer;
use crate::test_server::next_db_name;
use crate::test_server::next_user_name;
use crate::test_server::test_error::TestError;
use std::path::Path;

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn delete() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let user = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(user, user).await?;
    let status = server.api.admin_user_delete(user).await?;
    assert_eq!(status, 204);
    assert!(
        !server
            .api
            .admin_user_list()
            .await?
            .1
            .iter()
            .any(|u| u.username == *user)
    );
    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn delete_with_other() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let user = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_user_add(user, user).await?;
    server.api.admin_db_add(owner, db, DbKind::File).await?;
    server
        .api
        .admin_db_user_add(owner, db, user, DbUserRole::Write)
        .await?;
    server.api.admin_user_delete(owner).await?;
    assert!(
        !server
            .api
            .admin_user_list()
            .await?
            .1
            .iter()
            .any(|u| u.username == *owner)
    );
    assert!(!Path::new(&server.data_dir).join(owner).exists());
    server.api.user_login(user, user).await?;
    assert!(server.api.db_list().await?.1.is_empty());
    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn user_not_found() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    server.api.user_login(ADMIN, ADMIN).await?;
    let status = server
        .api
        .admin_user_delete("not_found")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 404);
    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn non_admin() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let user = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(user, user).await?;
    server.api.user_login(user, user).await?;
    let status = server.api.admin_user_delete(user).await.unwrap_err().status;
    assert_eq!(status, 401);
    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn no_token() -> Result<(), TestError> {
    let server = TestServer::new().await?;
    let status = server
        .api
        .admin_user_delete("user")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 401);
    Ok(())
}

#[cfg(feature = "api")]
pub fn test_defs() -> Vec<agdb::type_def::Type> {
    vec![
        __delete_type_def(),
        __delete_with_other_type_def(),
        __user_not_found_type_def(),
        __non_admin_type_def(),
        __no_token_type_def(),
    ]
}
