use agdb_api::AgdbApi;
use agdb_api::ReqwestClient;
use agdb_api::test_server::ADMIN;
use agdb_api::test_server::reqwest_client;
use agdb_api::test_server::test_cluster::create_cluster;
use agdb_api::test_server::test_cluster::wait_for_leader;
use agdb_api::test_server::test_error::TestError;
use agdb_api::test_server::wait_for_ready;

#[tokio::test]
async fn rebalance() -> Result<(), TestError> {
    let mut servers = create_cluster(3, false).await?;
    let mut leader = AgdbApi::new(
        ReqwestClient::with_client(reqwest_client()),
        &servers[0].address,
    );
    leader.user_login(ADMIN, ADMIN).await?;
    leader.admin_shutdown().await?;
    servers[0].wait().await?;

    let mut statuses = Vec::with_capacity(servers.len() - 1);

    for server in &servers[1..] {
        let status = wait_for_leader(&AgdbApi::new(
            ReqwestClient::with_client(reqwest_client()),
            &server.address,
        ))
        .await?;
        statuses.push(status);
    }

    for status in &statuses {
        assert_eq!(statuses[0], *status);
    }

    servers[0].restart()?;
    wait_for_ready(&leader).await?;

    statuses.clear();

    for server in &servers {
        let status = wait_for_leader(&AgdbApi::new(
            ReqwestClient::with_client(reqwest_client()),
            &server.address,
        ))
        .await?;
        statuses.push(status);
    }

    for status in &statuses {
        assert_eq!(statuses[0], *status);
    }

    Ok(())
}

#[tokio::test]
async fn status() -> Result<(), TestError> {
    agdb_api::tests::routes::cluster_test::status().await
}

#[tokio::test]
async fn admin_db_add() -> Result<(), TestError> {
    agdb_api::tests::routes::cluster_test::admin_db_add().await
}

#[tokio::test]
async fn admin_db_backup_restore() -> Result<(), TestError> {
    agdb_api::tests::routes::cluster_test::admin_db_backup_restore().await
}

#[tokio::test]
async fn admin_db_clear() -> Result<(), TestError> {
    agdb_api::tests::routes::cluster_test::admin_db_clear().await
}

#[tokio::test]
async fn admin_db_convert() -> Result<(), TestError> {
    agdb_api::tests::routes::cluster_test::admin_db_convert().await
}

#[tokio::test]
async fn admin_db_copy() -> Result<(), TestError> {
    agdb_api::tests::routes::cluster_test::admin_db_copy().await
}

#[tokio::test]
async fn admin_db_delete() -> Result<(), TestError> {
    agdb_api::tests::routes::cluster_test::admin_db_delete().await
}

#[tokio::test]
async fn admin_db_exec() -> Result<(), TestError> {
    agdb_api::tests::routes::cluster_test::admin_db_exec().await
}

#[tokio::test]
async fn admin_db_optimize() -> Result<(), TestError> {
    agdb_api::tests::routes::cluster_test::admin_db_optimize().await
}

#[tokio::test]
async fn admin_db_shrink_to_fit() -> Result<(), TestError> {
    agdb_api::tests::routes::cluster_test::admin_db_shrink_to_fit().await
}

#[tokio::test]
async fn admin_db_remove() -> Result<(), TestError> {
    agdb_api::tests::routes::cluster_test::admin_db_remove().await
}

#[tokio::test]
async fn admin_db_rename() -> Result<(), TestError> {
    agdb_api::tests::routes::cluster_test::admin_db_rename().await
}

#[tokio::test]
async fn admin_db_user_add_remove() -> Result<(), TestError> {
    agdb_api::tests::routes::cluster_test::admin_db_user_add_remove().await
}

#[tokio::test]
async fn admin_user_add() -> Result<(), TestError> {
    agdb_api::tests::routes::cluster_test::admin_user_add().await
}

#[tokio::test]
async fn admin_user_change_password() -> Result<(), TestError> {
    agdb_api::tests::routes::cluster_test::admin_user_change_password().await
}

#[tokio::test]
async fn admin_cluster_logout() -> Result<(), TestError> {
    agdb_api::tests::routes::cluster_test::admin_cluster_logout().await
}

#[tokio::test]
async fn admin_cluster_logout_all() -> Result<(), TestError> {
    agdb_api::tests::routes::cluster_test::admin_cluster_logout_all().await
}

#[tokio::test]
async fn admin_user_delete() -> Result<(), TestError> {
    agdb_api::tests::routes::cluster_test::admin_user_delete().await
}

#[tokio::test]
async fn db_add() -> Result<(), TestError> {
    agdb_api::tests::routes::cluster_test::db_add().await
}

#[tokio::test]
async fn db_backup() -> Result<(), TestError> {
    agdb_api::tests::routes::cluster_test::db_backup().await
}

#[tokio::test]
async fn db_clear() -> Result<(), TestError> {
    agdb_api::tests::routes::cluster_test::db_clear().await
}

#[tokio::test]
async fn db_convert() -> Result<(), TestError> {
    agdb_api::tests::routes::cluster_test::db_convert().await
}

#[tokio::test]
async fn db_copy() -> Result<(), TestError> {
    agdb_api::tests::routes::cluster_test::db_copy().await
}

#[tokio::test]
async fn db_delete() -> Result<(), TestError> {
    agdb_api::tests::routes::cluster_test::db_delete().await
}

#[tokio::test]
async fn db_exec() -> Result<(), TestError> {
    agdb_api::tests::routes::cluster_test::db_exec().await
}

#[tokio::test]
async fn db_optimize() -> Result<(), TestError> {
    agdb_api::tests::routes::cluster_test::db_optimize().await
}

#[tokio::test]
async fn db_shrink_to_fit() -> Result<(), TestError> {
    agdb_api::tests::routes::cluster_test::db_shrink_to_fit().await
}

#[tokio::test]
async fn db_remove() -> Result<(), TestError> {
    agdb_api::tests::routes::cluster_test::db_remove().await
}

#[tokio::test]
async fn db_rename() -> Result<(), TestError> {
    agdb_api::tests::routes::cluster_test::db_rename().await
}

#[tokio::test]
async fn db_user_add_remove() -> Result<(), TestError> {
    agdb_api::tests::routes::cluster_test::db_user_add_remove().await
}

#[tokio::test]
async fn user_change_password() -> Result<(), TestError> {
    agdb_api::tests::routes::cluster_test::user_change_password().await
}

#[tokio::test]
async fn user_login() -> Result<(), TestError> {
    agdb_api::tests::routes::cluster_test::user_login().await
}

#[tokio::test]
async fn cluster_logout() -> Result<(), TestError> {
    agdb_api::tests::routes::cluster_test::cluster_logout().await
}
