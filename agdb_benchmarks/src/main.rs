use crate::database::Database;
use bench_result::BenchResult;
use num_format::Locale;
use users::setup_users;

mod bench_error;
mod bench_result;
mod database;
mod users;
mod utilities;

pub(crate) const BENCH_DATABASE: &str = "db.agdb";
pub(crate) const LOCALE: Locale = Locale::es;
pub(crate) const USER_COUNT: u32 = 10_000;

fn main() -> BenchResult<()> {
    println!("Running agdb benchmark");
    println!("---");

    {
        let mut db = Database::new()?;
        setup_users(&mut db)?;
    }

    println!("---");
    Database::stat()
}
