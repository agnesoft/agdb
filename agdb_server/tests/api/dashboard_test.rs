use crate::runner::Test;
use crate::runner::tests;
use crate::test_server::ADMIN;
use crate::test_server::TestServer;
use crate::test_server::next_user_name;
use crate::test_server::test_error::TestError;
use agdb_api::DbUserRole;

async fn add_get_list() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let user = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(user, user).await?;
    server.api.user_login(user, user).await?;
    let status = server
        .api
        .dashboard_add(user, "ops", "{\"widgets\":[]}")
        .await?;
    assert_eq!(status, 201);
    server.api.dashboard_add(user, "alpha", "{}").await?;
    let (status, dashboard) = server.api.dashboard_get(user, "ops").await?;
    assert_eq!(status, 200);
    assert_eq!(dashboard.name, "ops");
    assert_eq!(dashboard.owner, *user);
    assert_eq!(dashboard.role, DbUserRole::Admin);
    assert_eq!(dashboard.config, "{\"widgets\":[]}");
    let (status, list) = server.api.dashboard_list().await?;
    assert_eq!(status, 200);
    // Every user lists the public dashboards of the other tests too, so only
    // the own ones are checked. The list is sorted by owner and name.
    let first = list
        .iter()
        .find(|d| d.owner == *user)
        .expect("own dashboard listed");
    assert_eq!(first.name, "alpha");
    assert!(list.iter().any(|d| d.owner == *user && d.name == "ops"));
    assert!(
        list.iter()
            .all(|d| d.owner != *user || d.name == "alpha" || d.name == "ops")
    );
    // The listing carries the time of the last save, not the config.
    assert!(
        list.iter()
            .all(|d| d.owner != *user || d.updated > 0 && d.role == DbUserRole::Admin)
    );
    Ok(())
}

async fn add_existing() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let user = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(user, user).await?;
    server.api.user_login(user, user).await?;
    server.api.dashboard_add(user, "ops", "{}").await?;
    let status = server
        .api
        .dashboard_add(user, "ops", "{}")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 409);
    Ok(())
}

async fn add_invalid_name() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let user = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(user, user).await?;
    server.api.user_login(user, user).await?;
    let status = server
        .api
        .dashboard_add(user, "bad!", "{}")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 400);
    let status = server
        .api
        .dashboard_add(user, ".ops", "{}")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 400);
    // Over the 256 KB config limit.
    let too_large = "x".repeat(256 * 1024 + 1);
    let status = server
        .api
        .dashboard_add(user, "big", &too_large)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 413);
    server.api.dashboard_add(user, "small", "{}").await?;
    let status = server
        .api
        .dashboard_save(user, "small", &too_large)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 413);
    Ok(())
}

async fn save_existing_only() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let user = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(user, user).await?;
    server.api.user_login(user, user).await?;
    let status = server
        .api
        .dashboard_save(user, "ops", "{\"v\":1}")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 404);
    server.api.dashboard_add(user, "ops", "{}").await?;
    let status = server.api.dashboard_save(user, "ops", "{\"v\":1}").await?;
    assert_eq!(status, 201);
    server.api.dashboard_save(user, "ops", "{\"v\":2}").await?;
    let (_, dashboard) = server.api.dashboard_get(user, "ops").await?;
    assert_eq!(dashboard.config, "{\"v\":2}");
    let (_, list) = server.api.dashboard_list().await?;
    // Every user lists the public dashboards of the other tests too.
    assert!(list.iter().any(|d| d.owner == *user && d.name == "ops"));
    assert!(list.iter().all(|d| d.owner != *user || d.name == "ops"));
    Ok(())
}

async fn rename() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let user = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(user, user).await?;
    server.api.user_login(user, user).await?;
    server.api.dashboard_add(user, "ops", "{}").await?;
    server.api.dashboard_add(user, "other", "{}").await?;
    let status = server.api.dashboard_rename(user, "ops", "sales").await?;
    assert_eq!(status, 201);
    let (_, dashboard) = server.api.dashboard_get(user, "sales").await?;
    assert_eq!(dashboard.config, "{}");
    let status = server
        .api
        .dashboard_get(user, "ops")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 404);
    let status = server
        .api
        .dashboard_rename(user, "sales", "other")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 409);
    let status = server
        .api
        .dashboard_rename(user, "missing", "x")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 404);
    let status = server
        .api
        .dashboard_rename(user, "sales", ".hidden")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 400);
    Ok(())
}

async fn remove() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let user = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(user, user).await?;
    server.api.user_login(user, user).await?;
    server.api.dashboard_add(user, "ops", "{}").await?;
    let status = server.api.dashboard_remove(user, "ops").await?;
    assert_eq!(status, 204);
    let (_, list) = server.api.dashboard_list().await?;
    // Every user lists the public dashboards of the other tests too.
    assert!(!list.iter().any(|d| d.owner == *user));
    let status = server
        .api
        .dashboard_remove(user, "ops")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 404);
    Ok(())
}

async fn other_user() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let user = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_user_add(user, user).await?;
    server.api.user_login(owner, owner).await?;
    server.api.dashboard_add(owner, "ops", "{}").await?;
    server.api.user_login(user, user).await?;
    let (_, list) = server.api.dashboard_list().await?;
    // Every user lists the public dashboards of the other tests too.
    assert!(!list.iter().any(|d| d.owner == *owner));
    let status = server
        .api
        .dashboard_get(owner, "ops")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 404);
    // Not even its existence is revealed by the other routes.
    let status = server
        .api
        .dashboard_save(owner, "ops", "{}")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 404);
    let status = server
        .api
        .dashboard_rename(owner, "ops", "x")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 404);
    let status = server
        .api
        .dashboard_remove(owner, "ops")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 404);
    let status = server
        .api
        .dashboard_public(owner, "ops", true)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 404);
    let status = server
        .api
        .dashboard_user_add(owner, "ops", user, DbUserRole::Admin)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 404);
    let status = server
        .api
        .dashboard_user_add(owner, "ops", owner, DbUserRole::Read)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 404);
    let status = server
        .api
        .dashboard_user_remove(owner, "ops", owner)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 404);
    let status = server
        .api
        .dashboard_user_remove(owner, "ops", user)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 404);
    let status = server
        .api
        .dashboard_user_list(owner, "ops")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 404);
    // Also before the config is checked.
    let too_large = "x".repeat(256 * 1024 + 1);
    let status = server
        .api
        .dashboard_save(owner, "ops", &too_large)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 404);
    let status = server
        .api
        .dashboard_add(owner, "mine", "{}")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 403);
    Ok(())
}

async fn removed_with_user() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let user = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(user, user).await?;
    server.api.user_login(user, user).await?;
    server.api.dashboard_add(user, "ops", "{}").await?;
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_delete(user).await?;
    server.api.admin_user_add(user, user).await?;
    server.api.user_login(user, user).await?;
    let (_, list) = server.api.dashboard_list().await?;
    // Every user lists the public dashboards of the other tests too.
    assert!(!list.iter().any(|d| d.owner == *user));
    Ok(())
}

async fn no_token() -> Result<(), TestError> {
    let server = TestServer::new().await?;
    let status = server.api.dashboard_list().await.unwrap_err().status;
    assert_eq!(status, 401);
    Ok(())
}

async fn share_read() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let user = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_user_add(user, user).await?;
    server.api.user_login(owner, owner).await?;
    server.api.dashboard_add(owner, "ops", "{}").await?;
    let status = server
        .api
        .dashboard_user_add(owner, "ops", user, DbUserRole::Read)
        .await?;
    assert_eq!(status, 201);
    server.api.user_login(user, user).await?;
    let (_, list) = server.api.dashboard_list().await?;
    // Every user lists the public dashboards of the other tests too.
    let listed = list
        .iter()
        .find(|d| d.owner == *owner)
        .expect("dashboard listed");
    assert_eq!(listed.owner, *owner);
    assert_eq!(listed.role, DbUserRole::Read);
    let (_, dashboard) = server.api.dashboard_get(owner, "ops").await?;
    assert_eq!(dashboard.role, DbUserRole::Read);
    let status = server
        .api
        .dashboard_save(owner, "ops", "{}")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 403);
    let status = server
        .api
        .dashboard_user_add(owner, "ops", user, DbUserRole::Admin)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 403);
    let (_, users) = server.api.dashboard_user_list(owner, "ops").await?;
    assert_eq!(users.len(), 2);
    assert!(
        users
            .iter()
            .any(|u| u.username == *owner && u.role == DbUserRole::Admin)
    );
    Ok(())
}

async fn share_write() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let user = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_user_add(user, user).await?;
    server.api.user_login(owner, owner).await?;
    server.api.dashboard_add(owner, "ops", "{}").await?;
    let status = server
        .api
        .dashboard_user_add(owner, "ops", user, DbUserRole::Write)
        .await?;
    assert_eq!(status, 201);
    server.api.user_login(user, user).await?;
    let other = &next_user_name();
    let status = server.api.dashboard_save(owner, "ops", "{\"v\":1}").await?;
    assert_eq!(status, 201);
    let (_, dashboard) = server.api.dashboard_get(owner, "ops").await?;
    assert_eq!(dashboard.config, "{\"v\":1}");
    let status = server
        .api
        .dashboard_user_add(owner, "ops", other, DbUserRole::Read)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 403);
    Ok(())
}

async fn share_admin() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let user = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_user_add(user, user).await?;
    server.api.user_login(owner, owner).await?;
    server.api.dashboard_add(owner, "ops", "{}").await?;
    let status = server
        .api
        .dashboard_user_add(owner, "ops", user, DbUserRole::Admin)
        .await?;
    assert_eq!(status, 201);
    server.api.user_login(user, user).await?;
    let other = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(other, other).await?;
    server.api.user_login(user, user).await?;
    let status = server
        .api
        .dashboard_user_add(owner, "ops", other, DbUserRole::Read)
        .await?;
    assert_eq!(status, 201);
    let status = server
        .api
        .dashboard_user_remove(owner, "ops", other)
        .await?;
    assert_eq!(status, 204);
    // Only the owner renames or removes, as with databases.
    let status = server
        .api
        .dashboard_rename(owner, "ops", "sales")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 403);
    let status = server
        .api
        .dashboard_remove(owner, "ops")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 403);
    // The owner keeps the admin role.
    let status = server
        .api
        .dashboard_user_add(owner, "ops", owner, DbUserRole::Read)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 403);
    let status = server
        .api
        .dashboard_user_remove(owner, "ops", owner)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 403);
    Ok(())
}

async fn share_leave() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let user = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_user_add(user, user).await?;
    server.api.user_login(owner, owner).await?;
    server.api.dashboard_add(owner, "ops", "{}").await?;
    let status = server
        .api
        .dashboard_user_add(owner, "ops", user, DbUserRole::Read)
        .await?;
    assert_eq!(status, 201);
    server.api.user_login(user, user).await?;
    let status = server.api.dashboard_user_remove(owner, "ops", user).await?;
    assert_eq!(status, 204);
    let (_, list) = server.api.dashboard_list().await?;
    // Every user lists the public dashboards of the other tests too.
    assert!(!list.iter().any(|d| d.owner == *owner));
    let status = server
        .api
        .dashboard_get(owner, "ops")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 404);
    Ok(())
}

async fn share_unknown_user() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let user = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(user, user).await?;
    server.api.user_login(user, user).await?;
    server.api.dashboard_add(user, "ops", "{}").await?;
    let status = server
        .api
        .dashboard_user_add(user, "ops", "missing_user", DbUserRole::Read)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 404);
    Ok(())
}

async fn home() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let user = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_user_add(user, user).await?;
    server.api.user_login(owner, owner).await?;
    server.api.dashboard_add(owner, "ops", "{}").await?;
    let status = server
        .api
        .dashboard_user_add(owner, "ops", user, DbUserRole::Read)
        .await?;
    assert_eq!(status, 201);
    server.api.user_login(user, user).await?;
    let (_, home) = server.api.user_home_dashboard().await?;
    assert!(home.is_none());
    let status = server
        .api
        .user_home_dashboard_set(owner, "missing")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 404);
    let status = server.api.user_home_dashboard_set(owner, "ops").await?;
    assert_eq!(status, 201);
    let (_, home) = server.api.user_home_dashboard().await?;
    let home = home.expect("the home dashboard is set");
    assert_eq!(home.owner, *owner);
    assert_eq!(home.name, "ops");

    // The home follows a rename by the owner.
    server.api.user_login(owner, owner).await?;
    server.api.dashboard_rename(owner, "ops", "sales").await?;
    server.api.user_login(user, user).await?;
    let (_, home) = server.api.user_home_dashboard().await?;
    let home = home.expect("the home dashboard is kept");
    assert_eq!(home.owner, *owner);
    assert_eq!(home.name, "sales");

    let status = server.api.user_home_dashboard_clear().await?;
    assert_eq!(status, 204);
    let (_, home) = server.api.user_home_dashboard().await?;
    assert!(home.is_none());

    // Losing the access, or the dashboard, clears the home.
    server.api.user_home_dashboard_set(owner, "sales").await?;
    server.api.user_login(owner, owner).await?;
    server
        .api
        .dashboard_user_remove(owner, "sales", user)
        .await?;
    server.api.user_login(user, user).await?;
    let (_, home) = server.api.user_home_dashboard().await?;
    assert!(home.is_none());

    server.api.dashboard_add(user, "mine", "{}").await?;
    server.api.user_home_dashboard_set(user, "mine").await?;
    server.api.dashboard_remove(user, "mine").await?;
    let (_, home) = server.api.user_home_dashboard().await?;
    assert!(home.is_none());
    Ok(())
}

/// Only dashboard admins remove other users; anyone may leave.
async fn share_remove_other_as_read() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let user = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_user_add(user, user).await?;
    server.api.user_login(owner, owner).await?;
    server.api.dashboard_add(owner, "ops", "{}").await?;
    let status = server
        .api
        .dashboard_user_add(owner, "ops", user, DbUserRole::Read)
        .await?;
    assert_eq!(status, 201);
    server.api.user_login(user, user).await?;
    let other = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(other, other).await?;
    server.api.user_login(owner, owner).await?;
    server
        .api
        .dashboard_user_add(owner, "ops", other, DbUserRole::Read)
        .await?;
    server.api.user_login(user, user).await?;
    let status = server
        .api
        .dashboard_user_remove(owner, "ops", other)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 403);
    let (_, users) = server.api.dashboard_user_list(owner, "ops").await?;
    assert_eq!(users.len(), 3);
    Ok(())
}

/// Only dashboard admins remove other users; anyone may leave.
async fn share_remove_other_as_write() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let user = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_user_add(user, user).await?;
    server.api.user_login(owner, owner).await?;
    server.api.dashboard_add(owner, "ops", "{}").await?;
    let status = server
        .api
        .dashboard_user_add(owner, "ops", user, DbUserRole::Write)
        .await?;
    assert_eq!(status, 201);
    server.api.user_login(user, user).await?;
    let other = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(other, other).await?;
    server.api.user_login(owner, owner).await?;
    server
        .api
        .dashboard_user_add(owner, "ops", other, DbUserRole::Read)
        .await?;
    server.api.user_login(user, user).await?;
    let status = server
        .api
        .dashboard_user_remove(owner, "ops", other)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 403);
    let (_, users) = server.api.dashboard_user_list(owner, "ops").await?;
    assert_eq!(users.len(), 3);
    Ok(())
}

async fn shared_not_in_db_list() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let user = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_user_add(user, user).await?;
    server.api.user_login(owner, owner).await?;
    server.api.dashboard_add(owner, "ops", "{}").await?;
    let status = server
        .api
        .dashboard_user_add(owner, "ops", user, DbUserRole::Admin)
        .await?;
    assert_eq!(status, 201);
    server.api.user_login(user, user).await?;
    let (_, list) = server.api.db_list().await?;
    assert!(list.is_empty());
    Ok(())
}

async fn public_owner_only() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let member = &next_user_name();
    let viewer = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_user_add(member, member).await?;
    server.api.admin_user_add(viewer, viewer).await?;
    server.api.user_login(owner, owner).await?;
    server.api.dashboard_add(owner, "ops", "{}").await?;
    server
        .api
        .dashboard_user_add(owner, "ops", member, DbUserRole::Read)
        .await?;
    let status = server.api.dashboard_public(owner, "ops", true).await?;
    assert_eq!(status, 201);
    server
        .api
        .dashboard_user_add(owner, "ops", member, DbUserRole::Admin)
        .await?;
    // Not even a dashboard admin, and a user without a role does not see a
    // private dashboard at all.
    server.api.user_login(member, member).await?;
    let status = server
        .api
        .dashboard_public(owner, "ops", false)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 403);
    server.api.user_login(owner, owner).await?;
    server.api.dashboard_add(owner, "private", "{}").await?;
    server.api.user_login(viewer, viewer).await?;
    let status = server
        .api
        .dashboard_public(owner, "private", true)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 404);
    let status = server
        .api
        .dashboard_public(owner, "ops", false)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 403);
    Ok(())
}

async fn public_view() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let member = &next_user_name();
    let viewer = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_user_add(member, member).await?;
    server.api.admin_user_add(viewer, viewer).await?;
    server.api.user_login(owner, owner).await?;
    server.api.dashboard_add(owner, "ops", "{}").await?;
    server
        .api
        .dashboard_user_add(owner, "ops", member, DbUserRole::Read)
        .await?;
    let status = server.api.dashboard_public(owner, "ops", true).await?;
    assert_eq!(status, 201);
    server.api.user_login(viewer, viewer).await?;
    let (_, list) = server.api.dashboard_list().await?;
    // Every user lists the public dashboards of the other tests too.
    let listed = list
        .iter()
        .find(|d| d.owner == *owner)
        .expect("dashboard listed");
    assert_eq!(listed.owner, *owner);
    assert_eq!(listed.role, DbUserRole::Read);
    assert!(listed.public);
    assert!(!listed.member);
    let (_, dashboard) = server.api.dashboard_get(owner, "ops").await?;
    assert_eq!(dashboard.role, DbUserRole::Read);
    assert!(dashboard.public);
    let status = server
        .api
        .dashboard_save(owner, "ops", "{}")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 403);
    // Who it is shared with is for its users only.
    let status = server
        .api
        .dashboard_user_list(owner, "ops")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 403);

    server.api.user_login(member, member).await?;
    let (_, list) = server.api.dashboard_list().await?;
    // Every user lists the public dashboards of the other tests too.
    let listed = list
        .iter()
        .find(|d| d.owner == *owner)
        .expect("dashboard listed");
    assert!(listed.public);
    assert!(listed.member);
    let (_, users) = server.api.dashboard_user_list(owner, "ops").await?;
    assert_eq!(users.len(), 2);

    server.api.user_login(owner, owner).await?;
    let (_, list) = server.api.dashboard_list().await?;
    // Every user lists the public dashboards of the other tests too.
    let listed = list
        .iter()
        .find(|d| d.owner == *owner)
        .expect("dashboard listed");
    assert_eq!(listed.role, DbUserRole::Admin);
    assert!(listed.public);
    assert!(listed.member);
    // A rename keeps it public.
    server.api.dashboard_rename(owner, "ops", "sales").await?;
    server.api.user_login(viewer, viewer).await?;
    let (_, dashboard) = server.api.dashboard_get(owner, "sales").await?;
    assert!(dashboard.public);
    Ok(())
}

async fn public_private_again() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let member = &next_user_name();
    let viewer = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_user_add(member, member).await?;
    server.api.admin_user_add(viewer, viewer).await?;
    server.api.user_login(owner, owner).await?;
    server.api.dashboard_add(owner, "ops", "{}").await?;
    server
        .api
        .dashboard_user_add(owner, "ops", member, DbUserRole::Read)
        .await?;
    let status = server.api.dashboard_public(owner, "ops", true).await?;
    assert_eq!(status, 201);
    server.api.user_login(member, member).await?;
    server.api.user_home_dashboard_set(owner, "ops").await?;
    server.api.user_login(viewer, viewer).await?;
    server.api.user_home_dashboard_set(owner, "ops").await?;
    server.api.user_login(owner, owner).await?;
    server.api.dashboard_public(owner, "ops", false).await?;

    // Only users with a role keep it.
    server.api.user_login(viewer, viewer).await?;
    let (_, home) = server.api.user_home_dashboard().await?;
    assert!(home.is_none());
    let (_, list) = server.api.dashboard_list().await?;
    // Every user lists the public dashboards of the other tests too.
    assert!(!list.iter().any(|d| d.owner == *owner));
    let status = server
        .api
        .dashboard_get(owner, "ops")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 404);
    server.api.user_login(member, member).await?;
    let (_, home) = server.api.user_home_dashboard().await?;
    assert!(home.is_some());
    let (_, dashboard) = server.api.dashboard_get(owner, "ops").await?;
    assert!(!dashboard.public);
    Ok(())
}

async fn public_leave_keeps_home() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let member = &next_user_name();
    let viewer = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_user_add(member, member).await?;
    server.api.admin_user_add(viewer, viewer).await?;
    server.api.user_login(owner, owner).await?;
    server.api.dashboard_add(owner, "ops", "{}").await?;
    server
        .api
        .dashboard_user_add(owner, "ops", member, DbUserRole::Read)
        .await?;
    let status = server.api.dashboard_public(owner, "ops", true).await?;
    assert_eq!(status, 201);
    server.api.user_login(member, member).await?;
    server.api.user_home_dashboard_set(owner, "ops").await?;
    server
        .api
        .dashboard_user_remove(owner, "ops", member)
        .await?;
    // Still public, so still listed (without a role) and still the home.
    let (_, list) = server.api.dashboard_list().await?;
    // Every user lists the public dashboards of the other tests too.
    let listed = list
        .iter()
        .find(|d| d.owner == *owner)
        .expect("dashboard listed");
    assert_eq!(listed.role, DbUserRole::Read);
    assert!(!listed.member);
    let (_, home) = server.api.user_home_dashboard().await?;
    assert!(home.is_some());
    // Leaving again without a role changes nothing.
    let status = server
        .api
        .dashboard_user_remove(owner, "ops", member)
        .await?;
    assert_eq!(status, 204);
    let (_, list) = server.api.dashboard_list().await?;
    // Every user lists the public dashboards of the other tests too.
    let listed = list
        .iter()
        .find(|d| d.owner == *owner)
        .expect("dashboard listed");
    assert_eq!(listed.role, DbUserRole::Read);
    assert!(!listed.member);
    let (_, home) = server.api.user_home_dashboard().await?;
    assert!(home.is_some());
    // Removing someone else still needs the admin role.
    let status = server
        .api
        .dashboard_user_remove(owner, "ops", owner)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 403);
    Ok(())
}

pub(super) const TESTS: &[Test] = tests![
    add_get_list,
    add_existing,
    add_invalid_name,
    save_existing_only,
    rename,
    remove,
    other_user,
    removed_with_user,
    no_token,
    share_read,
    share_write,
    share_admin,
    share_leave,
    share_unknown_user,
    home,
    share_remove_other_as_read,
    share_remove_other_as_write,
    shared_not_in_db_list,
    public_owner_only,
    public_view,
    public_private_again,
    public_leave_keeps_home,
];
