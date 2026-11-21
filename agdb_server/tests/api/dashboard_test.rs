use crate::test_server::ADMIN;
use crate::test_server::TestServer;
use crate::test_server::next_user_name;
use crate::test_server::test_error::TestError;

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn add_get_list() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let user = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(user, user).await?;
    server.api.user_login(user, user).await?;
    let status = server.api.dashboard_add("ops", "{\"widgets\":[]}").await?;
    assert_eq!(status, 201);
    server.api.dashboard_add("alpha", "{}").await?;
    let (status, dashboard) = server.api.dashboard_get("ops").await?;
    assert_eq!(status, 200);
    assert_eq!(dashboard.name, "ops");
    assert_eq!(dashboard.config, "{\"widgets\":[]}");
    let (status, list) = server.api.dashboard_list().await?;
    assert_eq!(status, 200);
    let names: Vec<&str> = list.iter().map(|d| d.name.as_str()).collect();
    assert_eq!(names, vec!["alpha", "ops"]);
    // The listing carries the time of the last save, not the config.
    assert!(list.iter().all(|d| d.updated > 0));
    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn add_existing() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let user = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(user, user).await?;
    server.api.user_login(user, user).await?;
    server.api.dashboard_add("ops", "{}").await?;
    let status = server
        .api
        .dashboard_add("ops", "{}")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 409);
    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn add_invalid_name() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let user = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(user, user).await?;
    server.api.user_login(user, user).await?;
    let status = server
        .api
        .dashboard_add("list", "{}")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 400);
    let status = server
        .api
        .dashboard_add(".ops", "{}")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 400);
    // Over the 256 KB config limit.
    let too_large = "x".repeat(256 * 1024 + 1);
    let status = server
        .api
        .dashboard_add("big", &too_large)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 413);
    let status = server
        .api
        .dashboard_save("big", &too_large)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 413);
    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn save_upserts() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let user = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(user, user).await?;
    server.api.user_login(user, user).await?;
    let status = server.api.dashboard_save("ops", "{\"v\":1}").await?;
    assert_eq!(status, 201);
    server.api.dashboard_save("ops", "{\"v\":2}").await?;
    let (_, dashboard) = server.api.dashboard_get("ops").await?;
    assert_eq!(dashboard.config, "{\"v\":2}");
    let (_, list) = server.api.dashboard_list().await?;
    assert_eq!(list.len(), 1);
    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn rename() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let user = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(user, user).await?;
    server.api.user_login(user, user).await?;
    server.api.dashboard_add("ops", "{}").await?;
    server.api.dashboard_add("other", "{}").await?;
    let status = server.api.dashboard_rename("ops", "sales").await?;
    assert_eq!(status, 201);
    let (_, dashboard) = server.api.dashboard_get("sales").await?;
    assert_eq!(dashboard.config, "{}");
    let status = server.api.dashboard_get("ops").await.unwrap_err().status;
    assert_eq!(status, 404);
    let status = server
        .api
        .dashboard_rename("sales", "other")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 409);
    let status = server
        .api
        .dashboard_rename("missing", "x")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 404);
    let status = server
        .api
        .dashboard_rename("sales", "list")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 400);
    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn remove() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let user = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(user, user).await?;
    server.api.user_login(user, user).await?;
    server.api.dashboard_add("ops", "{}").await?;
    let status = server.api.dashboard_remove("ops").await?;
    assert_eq!(status, 204);
    let (_, list) = server.api.dashboard_list().await?;
    assert!(list.is_empty());
    let status = server.api.dashboard_remove("ops").await.unwrap_err().status;
    assert_eq!(status, 404);
    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn other_user() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let user = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_user_add(user, user).await?;
    server.api.user_login(owner, owner).await?;
    server.api.dashboard_add("ops", "{}").await?;
    server.api.user_login(user, user).await?;
    let (_, list) = server.api.dashboard_list().await?;
    assert!(list.is_empty());
    let status = server.api.dashboard_get("ops").await.unwrap_err().status;
    assert_eq!(status, 404);
    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn removed_with_user() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let user = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(user, user).await?;
    server.api.user_login(user, user).await?;
    server.api.dashboard_add("ops", "{}").await?;
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_delete(user).await?;
    server.api.admin_user_add(user, user).await?;
    server.api.user_login(user, user).await?;
    let (_, list) = server.api.dashboard_list().await?;
    assert!(list.is_empty());
    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn no_token() -> Result<(), TestError> {
    let server = TestServer::new().await?;
    let status = server.api.dashboard_list().await.unwrap_err().status;
    assert_eq!(status, 401);
    Ok(())
}

#[cfg(feature = "api")]
pub fn test_defs() -> Vec<agdb::type_def::Type> {
    vec![
        __add_get_list_type_def(),
        __add_existing_type_def(),
        __add_invalid_name_type_def(),
        __save_upserts_type_def(),
        __rename_type_def(),
        __remove_type_def(),
        __other_user_type_def(),
        __removed_with_user_type_def(),
        __no_token_type_def(),
    ]
}
