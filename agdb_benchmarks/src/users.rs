use crate::bench_result::BenchResult;
use crate::config::Config;
use crate::database::BENCHMARK_DATABASE;
use crate::database::BENCHMARK_USERNAME;
use crate::database::Database;
use crate::database::ServerDatabase;
use crate::queries::BenchUser;
use crate::utilities::format_duration;
use crate::utilities::measured;
use crate::utilities::measured_async;
use crate::utilities::print_flush;
use agdb::QueryBuilder;
use agdb::StorageData;
use agdb_api::AgdbApi;
use agdb_api::DbUserRole;
use agdb_api::ReqwestClient;
use num_format::ToFormattedString;

pub(crate) fn setup_users<S: StorageData>(
    db: &mut Database<S>,
    config: &Config,
) -> BenchResult<()> {
    let mut db = db.0.write()?;
    let padding = config.padding as usize;
    let cell_padding = config.cell_padding as usize;
    let user_count = config.user_count();

    print_flush(format!(
        "{:<padding$} | {:<cell_padding$} | {:<cell_padding$} | {:<cell_padding$} | {:<cell_padding$} |",
        "Creating users",
        1,
        1,
        user_count.to_formatted_string(&config.locale),
        user_count.to_formatted_string(&config.locale)
    ));

    let duration = measured(|| {
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
    })?;

    print_flush(format!(
        " {:cell_padding$} | {:cell_padding$} | {:cell_padding$} | {:cell_padding$}\n",
        "-",
        format_duration(duration / (user_count as u32), config.locale),
        "-",
        format_duration(duration, config.locale)
    ));

    Ok(())
}

pub(crate) async fn setup_server_users(
    db: &mut ServerDatabase,
    config: &Config,
) -> BenchResult<()> {
    let mut admin_api = AgdbApi::new(ReqwestClient::new(), db.address());
    admin_api.user_login("admin", "admin").await?;
    ensure_users_exist(&admin_api, config).await?;

    let padding = config.padding as usize;
    let cell_padding = config.cell_padding as usize;
    let total_users = config.posters.count
        + config.commenters.count
        + config.post_readers.count
        + config.comment_readers.count;

    print_flush(format!(
        "{:<padding$} | {:<cell_padding$} | {:<cell_padding$} | {:<cell_padding$} | {:<cell_padding$} |",
        "Creating users",
        1,
        1,
        total_users.to_formatted_string(&config.locale),
        total_users.to_formatted_string(&config.locale)
    ));

    let duration = measured_async(ensure_database_users_in_db(&admin_api, config)).await?;

    print_flush(format!(
        " {:cell_padding$} | {:cell_padding$} | {:cell_padding$} | {:cell_padding$}\n",
        "-",
        format_duration(duration / (total_users as u32), config.locale),
        "-",
        format_duration(duration, config.locale)
    ));

    Ok(())
}

pub(crate) async fn setup_server_bench_users(
    db: &ServerDatabase,
    config: &Config,
) -> BenchResult<()> {
    let padding = config.padding as usize;
    let cell_padding = config.cell_padding as usize;
    let user_count = config.user_count();

    print_flush(format!(
        "{:<padding$} | {:<cell_padding$} | {:<cell_padding$} | {:<cell_padding$} | {:<cell_padding$} |",
        "Adding bench users",
        1,
        1,
        user_count.to_formatted_string(&config.locale),
        user_count.to_formatted_string(&config.locale)
    ));

    let duration = measured_async(insert_bench_users(db, config)).await?;

    print_flush(format!(
        " {:cell_padding$} | {:cell_padding$} | {:cell_padding$} | {:cell_padding$}\n",
        "-",
        format_duration(duration / (user_count as u32), config.locale),
        "-",
        format_duration(duration, config.locale)
    ));

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
