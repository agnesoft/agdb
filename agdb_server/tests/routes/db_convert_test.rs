use crate::next_db_name;
use crate::next_user_name;
use crate::TestServer;
use crate::ADMIN;
use agdb::QueryBuilder;
use agdb_api::DbType;
use agdb_api::DbUserRole;

#[tokio::test]
async fn memory_to_mapped() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.user_login(owner, owner).await?;
    server.api.db_add(owner, db, DbType::Memory).await?;
    let status = server.api.db_convert(owner, db, DbType::Mapped).await?;
    assert_eq!(status, 201);
    let list = server.api.db_list().await?.1;
    assert_eq!(list[0].db_type, DbType::Mapped);

    Ok(())
}

#[tokio::test]
async fn same_type() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.user_login(owner, owner).await?;
    server.api.db_add(owner, db, DbType::Memory).await?;
    let status = server.api.db_convert(owner, db, DbType::Memory).await?;
    assert_eq!(status, 201);
    let list = server.api.db_list().await?.1;
    assert_eq!(list[0].db_type, DbType::Memory);

    Ok(())
}

#[tokio::test]
async fn file_to_memory() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.user_login(owner, owner).await?;
    server.api.db_add(owner, db, DbType::File).await?;
    server
        .api
        .db_exec_mut(
            owner,
            db,
            &[QueryBuilder::insert().nodes().count(1).query().into()],
        )
        .await?;
    let status = server.api.db_convert(owner, db, DbType::Memory).await?;
    assert_eq!(status, 201);
    let list = server.api.db_list().await?.1;
    assert_eq!(list[0].db_type, DbType::Memory);
    let nodes = server
        .api
        .db_exec(
            owner,
            db,
            &[QueryBuilder::select().node_count().query().into()],
        )
        .await?
        .1[0]
        .elements[0]
        .values[0]
        .value
        .to_u64()?;
    assert_eq!(nodes, 1);

    Ok(())
}

#[tokio::test]
async fn db_not_found() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.user_login(owner, owner).await?;
    let status = server
        .api
        .db_convert(owner, db, DbType::Mapped)
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
    let user = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_user_add(user, user).await?;
    server.api.admin_db_add(owner, db, DbType::Memory).await?;
    server
        .api
        .admin_db_user_add(owner, db, user, DbUserRole::Write)
        .await?;
    server.api.user_login(user, user).await?;
    let status = server
        .api
        .db_convert(owner, db, DbType::Mapped)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 403);
    Ok(())
}

#[tokio::test]
async fn no_token() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let status = server
        .api
        .db_convert("owner", "db", DbType::Memory)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 401);

    Ok(())
}
