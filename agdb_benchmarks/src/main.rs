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
pub(crate) const LOCALE: Locale = Locale::cs;
pub(crate) const USER_COUNT: u32 = 10_000;
pub(crate) const PADDING: usize = 25;
pub(crate) const CELL_PADDING: usize = 10;

fn main() -> BenchResult<()> {
    println!("Running agdb benchmark");
    println!("---");

    let mut db = Database::new()?;
    setup_users(&mut db)?;
    db.stat()
}
