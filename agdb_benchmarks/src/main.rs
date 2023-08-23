use crate::database::Database;
use agdb::QueryBuilder;
use bench_result::BenchResult;

mod bench_error;
mod bench_result;
mod database;

fn main() -> BenchResult<()> {
    let db = Database::new()?;

    db.0.write()?
        .exec_mut(&QueryBuilder::insert().nodes().count(1).query())?;

    Ok(())
}
