use crate::bench_result::BenchResult;
use crate::config::Config;
use crate::database::Database;
use crate::utilities::format_duration;
use crate::utilities::measured;
use crate::utilities::print_flush;
use agdb::QueryBuilder;
use agdb::StorageData;
use agdb::UserValue;
use num_format::ToFormattedString;

#[derive(UserValue)]
struct User {
    name: String,
    email: String,
}

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
                        &QueryBuilder::insert()
                            .nodes()
                            .values(&User {
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
                &QueryBuilder::insert()
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
