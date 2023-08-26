use crate::bench_result::BenchResult;
use crate::database::Database;
use crate::utilities::format_duration;
use crate::utilities::measured;
use crate::utilities::print_flush;
use crate::CELL_PADDING;
use crate::LOCALE;
use crate::PADDING;
use crate::USER_COUNT;
use agdb::QueryBuilder;
use agdb::UserValue;
use num_format::ToFormattedString;

#[derive(UserValue)]
struct User {
    name: String,
    email: String,
}

pub(crate) fn setup_users(db: &mut Database) -> BenchResult<()> {
    let mut db = db.0.write()?;

    print_flush(format!(
        "{:PADDING$} | {:CELL_PADDING$} |",
        "Creating users",
        USER_COUNT.to_formatted_string(&LOCALE)
    ));

    let duration = measured(|| {
        db.transaction_mut(|t| {
            let mut user_ids = vec![];

            for i in 0..USER_COUNT {
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
        " {:CELL_PADDING$} | {:CELL_PADDING$} | {:CELL_PADDING$} | {:CELL_PADDING$}\n",
        "",
        format_duration(duration / USER_COUNT),
        "",
        format_duration(duration)
    ));

    Ok(())
}
