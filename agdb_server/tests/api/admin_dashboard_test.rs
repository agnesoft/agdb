use crate::runner::Test;
use crate::runner::tests;
use crate::test_server::ADMIN;
use crate::test_server::TestServer;
use crate::test_server::next_user_name;
use crate::test_server::test_error::TestError;
use agdb_api::DbUserRole;

async fn non_admin() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let user = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_user_add(user, user).await?;
    server.api.user_login(owner, owner).await?;
    server.api.dashboard_add(owner, "ops", "{}").await?;
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.user_login(owner, owner).await?;
    let status = server.api.admin_dashboard_list().await.unwrap_err().status;
    assert_eq!(status, 401);
    let status = server
        .api
        .admin_dashboard_user_add(owner, "ops", user, DbUserRole::Read)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 401);
    Ok(())
}

async fn list_get_save() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let user = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_user_add(user, user).await?;
    server.api.user_login(owner, owner).await?;
    server.api.dashboard_add(owner, "ops", "{}").await?;
    server.api.user_login(ADMIN, ADMIN).await?;
    let (status, list) = server.api.admin_dashboard_list().await?;
    assert_eq!(status, 200);
    let listed = list
        .iter()
        .find(|d| d.owner == *owner && d.name == "ops")
        .expect("the dashboard is listed");
    assert_eq!(listed.role, DbUserRole::Admin);
    assert!(!listed.public);
    assert!(listed.member);
    let status = server
        .api
        .admin_dashboard_save(owner, "ops", "{\"v\":1}")
        .await?;
    assert_eq!(status, 201);
    let (_, dashboard) = server.api.admin_dashboard_get(owner, "ops").await?;
    assert_eq!(dashboard.config, "{\"v\":1}");
    assert_eq!(dashboard.role, DbUserRole::Admin);
    assert!(!dashboard.public);
    let status = server
        .api
        .admin_dashboard_get(owner, "missing")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 404);
    let status = server
        .api
        .admin_dashboard_save(owner, "missing", "{}")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 404);
    Ok(())
}

async fn add() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let user = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_user_add(user, user).await?;
    server.api.user_login(owner, owner).await?;
    server.api.dashboard_add(owner, "ops", "{}").await?;
    server.api.user_login(ADMIN, ADMIN).await?;
    let status = server.api.admin_dashboard_add(owner, "added", "{}").await?;
    assert_eq!(status, 201);
    let status = server
        .api
        .admin_dashboard_add(owner, "added", "{}")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 409);
    let status = server
        .api
        .admin_dashboard_add("missing_user", "added", "{}")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 404);
    // The owner gets the admin role on it.
    server.api.user_login(owner, owner).await?;
    let (_, dashboard) = server.api.dashboard_get(owner, "added").await?;
    assert_eq!(dashboard.role, DbUserRole::Admin);
    Ok(())
}

async fn rename_remove() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let user = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_user_add(user, user).await?;
    server.api.user_login(owner, owner).await?;
    server.api.dashboard_add(owner, "ops", "{}").await?;
    server.api.user_login(ADMIN, ADMIN).await?;
    let status = server
        .api
        .admin_dashboard_rename(owner, "ops", "sales")
        .await?;
    assert_eq!(status, 201);
    let status = server
        .api
        .admin_dashboard_rename(owner, "missing", "x")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 404);
    let status = server.api.admin_dashboard_remove(owner, "sales").await?;
    assert_eq!(status, 204);
    server.api.user_login(owner, owner).await?;
    let (_, list) = server.api.dashboard_list().await?;
    // Every user lists the public dashboards of the other tests too.
    assert!(!list.iter().any(|d| d.owner == *owner));
    Ok(())
}

async fn public() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let viewer = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_user_add(viewer, viewer).await?;
    server.api.admin_dashboard_add(owner, "ops", "{}").await?;
    let status = server
        .api
        .admin_dashboard_public(owner, "ops", true)
        .await?;
    assert_eq!(status, 201);
    let (_, dashboard) = server.api.admin_dashboard_get(owner, "ops").await?;
    assert!(dashboard.public);
    server.api.user_login(viewer, viewer).await?;
    let (_, list) = server.api.dashboard_list().await?;
    // Every user lists the public dashboards of the other tests too.
    assert!(list.iter().any(|d| d.owner == *owner && d.name == "ops"));
    assert!(list.iter().all(|d| d.owner != *owner || d.name == "ops"));
    server.api.user_login(ADMIN, ADMIN).await?;
    server
        .api
        .admin_dashboard_public(owner, "ops", false)
        .await?;
    server.api.user_login(viewer, viewer).await?;
    let (_, list) = server.api.dashboard_list().await?;
    // Every user lists the public dashboards of the other tests too.
    assert!(!list.iter().any(|d| d.owner == *owner));
    Ok(())
}

async fn users() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let user = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_user_add(user, user).await?;
    server.api.user_login(owner, owner).await?;
    server.api.dashboard_add(owner, "ops", "{}").await?;
    server.api.user_login(ADMIN, ADMIN).await?;
    let status = server
        .api
        .admin_dashboard_user_add(owner, "ops", user, DbUserRole::Write)
        .await?;
    assert_eq!(status, 201);
    let (_, users) = server.api.admin_dashboard_user_list(owner, "ops").await?;
    assert_eq!(users.len(), 2);
    assert!(
        users
            .iter()
            .any(|u| u.username == *user && u.role == DbUserRole::Write)
    );
    // The owner keeps the admin role and stays.
    let status = server
        .api
        .admin_dashboard_user_add(owner, "ops", owner, DbUserRole::Read)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 403);
    let status = server
        .api
        .admin_dashboard_user_remove(owner, "ops", owner)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 403);
    let status = server
        .api
        .admin_dashboard_user_add(owner, "ops", "missing_user", DbUserRole::Read)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 404);
    let status = server
        .api
        .admin_dashboard_user_remove(owner, "ops", user)
        .await?;
    assert_eq!(status, 204);
    let (_, users) = server.api.admin_dashboard_user_list(owner, "ops").await?;
    assert_eq!(users.len(), 1);
    Ok(())
}

pub(super) const TESTS: &[Test] =
    tests![non_admin, list_get_save, add, rename_remove, public, users,];
