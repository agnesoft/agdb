use crate::database::Database;
use agdb::DbId;
use agdb::QueryBuilder;
use agdb::UserValue;
use bench_result::BenchResult;

mod bench_error;
mod bench_result;
mod database;

const USER_COUNT: u64 = 10000;

#[derive(UserValue)]
struct User {
    name: String,
    email: String,
}

fn setup_users(db: &mut Database) -> BenchResult<()> {
    db.0.write()?
        .exec_mut(&QueryBuilder::insert().nodes().aliases("users").query());

    let mut user_values: Vec<User>;

    db.0.write()?
        .exec_mut(&QueryBuilder::insert().nodes().values(&user_values).query())?;

    Ok(())
}

fn main() -> BenchResult<()> {
    let mut db = Database::new()?;

    setup_users(&mut db)?;

    Ok(())
}
