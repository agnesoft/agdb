use crate::bench_result::BenchResult;
use crate::config::Config;
use crate::database::BENCHMARK_DATABASE;
use crate::database::BENCHMARK_USERNAME;
use crate::database::Database;
use crate::database::ServerDatabase;
use crate::database::admin_api;
use crate::queries::BenchUser;
use agdb::QueryBuilder;
use agdb::StorageData;
use agdb_api::AgdbApi;
use agdb_api::DbUserRole;
use agdb_api::ReqwestClient;

pub(crate) fn setup_users<S: StorageData>(
    db: &mut Database<S>,
    config: &Config,
) -> BenchResult<()> {
    let mut db = db.0.write()?;
    let user_count = config.user_count();

    db.transaction_mut(|t| {
        let mut user_ids = vec![];

        for i in 0..user_count {
            user_ids.push(
                t.exec_mut(
                    QueryBuilder::insert()
                        .nodes()
                        .values(BenchUser {
                            name: format!("u{i}"),
                            email: format!("u{i}@a.com"),
                        })
                        .query(),
                )?
                .elements[0]
                    .id,
            );
        }

        t.exec_mut(
            QueryBuilder::insert()
                .edges()
                .from("users")
                .to(user_ids)
                .query(),
        )
    })?;

    Ok(())
}

pub(crate) async fn setup_server_users(
    db: &mut ServerDatabase,
    config: &Config,
) -> BenchResult<()> {
    let mut admin_api = admin_api(db.client().clone(), db.address());
    admin_api.user_login("admin", "admin").await?;
    ensure_users_exist(&admin_api, config).await?;
    ensure_database_users_in_db(&admin_api, config).await?;

    Ok(())
}

pub(crate) async fn setup_server_bench_users(
    db: &ServerDatabase,
    config: &Config,
) -> BenchResult<()> {
    insert_bench_users(db, config).await?;

    Ok(())
}

pub(crate) fn post_writer_username(index: u64) -> String {
    format!("postwriter_{index}")
}

pub(crate) fn comment_writer_username(index: u64) -> String {
    format!("commentwriter_{index}")
}

pub(crate) fn post_reader_username(index: u64) -> String {
    format!("postreader_{index}")
}

pub(crate) fn comment_reader_username(index: u64) -> String {
    format!("commentreader_{index}")
}

pub(crate) fn benchmark_password(username: &str) -> String {
    username.to_string()
}

async fn ensure_users_exist(
    admin_api: &AgdbApi<ReqwestClient>,
    config: &Config,
) -> BenchResult<()> {
    for index in 0..config.posters.count {
        ensure_user_exists(admin_api, &post_writer_username(index)).await?;
    }

    for index in 0..config.commenters.count {
        ensure_user_exists(admin_api, &comment_writer_username(index)).await?;
    }

    for index in 0..config.post_readers.count {
        ensure_user_exists(admin_api, &post_reader_username(index)).await?;
    }

    for index in 0..config.comment_readers.count {
        ensure_user_exists(admin_api, &comment_reader_username(index)).await?;
    }

    Ok(())
}

async fn ensure_database_users_in_db(
    admin_api: &AgdbApi<ReqwestClient>,
    config: &Config,
) -> BenchResult<()> {
    for index in 0..config.posters.count {
        ensure_user_db_role(admin_api, &post_writer_username(index), DbUserRole::Write).await?;
    }

    for index in 0..config.commenters.count {
        ensure_user_db_role(
            admin_api,
            &comment_writer_username(index),
            DbUserRole::Write,
        )
        .await?;
    }

    for index in 0..config.post_readers.count {
        ensure_user_db_role(admin_api, &post_reader_username(index), DbUserRole::Read).await?;
    }

    for index in 0..config.comment_readers.count {
        ensure_user_db_role(admin_api, &comment_reader_username(index), DbUserRole::Read).await?;
    }

    Ok(())
}

async fn ensure_user_exists(admin_api: &AgdbApi<ReqwestClient>, username: &str) -> BenchResult<()> {
    let password = benchmark_password(username);
    match admin_api.admin_user_add(username, &password).await {
        Ok(_) => {}
        Err(error) if error.status == 463 => {}
        Err(error) => return Err(error.into()),
    }

    Ok(())
}

async fn ensure_user_db_role(
    admin_api: &AgdbApi<ReqwestClient>,
    username: &str,
    role: DbUserRole,
) -> BenchResult<()> {
    admin_api
        .admin_db_user_add(BENCHMARK_USERNAME, BENCHMARK_DATABASE, username, role)
        .await?;

    Ok(())
}

async fn insert_bench_users(db: &ServerDatabase, config: &Config) -> BenchResult<()> {
    let mut users = Vec::new();

    for index in 0..(config.posters.count + config.commenters.count) {
        users.push(BenchUser {
            name: format!("user_{index}"),
            email: format!("user_{index}@a.com"),
        });
    }

    db.exec_mut(&[
        QueryBuilder::insert().elements(&users).query().into(),
        QueryBuilder::insert()
            .edges()
            .from("users")
            .to(":0")
            .query()
            .into(),
    ])
    .await?;

    Ok(())
}
