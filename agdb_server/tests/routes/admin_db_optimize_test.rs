use crate::next_db_name;
use crate::next_user_name;
use crate::TestCluster;
use crate::TestServer;
use crate::ADMIN;
use agdb::QueryBuilder;
use agdb_api::DbType;

#[tokio::test]
async fn optimize() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_db_add(owner, db, DbType::Mapped).await?;
    let queries = &vec![QueryBuilder::insert().nodes().count(100).query().into()];
    server.api.admin_db_exec_mut(owner, db, queries).await?;
    let original_size = server
        .api
        .admin_db_list()
        .await?
        .1
        .iter()
        .find(|d| d.name == format!("{owner}/{db}"))
        .unwrap()
        .size;
    let (status, db) = server.api.admin_db_optimize(owner, db).await?;
    assert_eq!(status, 200);
    assert!(db.size < original_size);
    Ok(())
}

#[tokio::test]
async fn db_not_found() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    server.api.user_login(ADMIN, ADMIN).await?;
    let status = server
        .api
        .admin_db_optimize("owner", "db")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 404);
    Ok(())
}

#[tokio::test]
async fn non_admin() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.user_login(owner, owner).await?;
    let status = server
        .api
        .admin_db_optimize(owner, "db")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 401);
    Ok(())
}

#[tokio::test]
async fn no_token() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let status = server
        .api
        .admin_db_optimize("owner", "db")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 401);
    Ok(())
}

#[tokio::test]
async fn cluster_exec() -> anyhow::Result<()> {
    let mut cluster = TestCluster::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    let client = cluster.apis.get_mut(1).unwrap();
    client.cluster_login(ADMIN, ADMIN).await?;
    client.admin_user_add(owner, owner).await?;
    client.admin_db_add(owner, db, DbType::Memory).await?;
    client
        .admin_db_exec_mut(
            owner,
            db,
            &[QueryBuilder::insert().nodes().count(100).query().into()],
        )
        .await?;
    let original_size = client
        .admin_db_list()
        .await?
        .1
        .iter()
        .find(|d| d.name == format!("{owner}/{db}"))
        .unwrap()
        .size;
    client.admin_db_optimize(owner, owner).await?;
    let server_db = client.admin_db_optimize(owner, db).await?.1;
    assert!(server_db.size < original_size);
    Ok(())
}
