use crate::database::Database;
use crate::writer::start_post_writers;
use bench_result::BenchResult;
use num_format::Locale;
use std::time::Duration;
use users::setup_users;

mod bench_error;
mod bench_result;
mod database;
mod users;
mod utilities;
mod writer;

pub(crate) const BENCH_DATABASE: &str = "db.agdb";
pub(crate) const LOCALE: Locale = Locale::cs;
pub(crate) const USER_COUNT: u32 = 10;
pub(crate) const WRITER_COUNT: u32 = 10;
pub(crate) const WRITE_POSTS: u32 = 10;
pub(crate) const WRITE_COMMENTS: u32 = 100;
pub(crate) const WRITE_DELAY: Duration = Duration::from_millis(10);
pub(crate) const PADDING: usize = 25;
pub(crate) const CELL_PADDING: usize = 10;
pub(crate) const POST_TITLE: &str = "Title of the testing post";
pub(crate) const POST_BODY: &str = "Body of the testing post should be longer than the title";
pub(crate) const COMMENT_BODY: &str = "This is a testing comment of a post.";

fn main() -> BenchResult<()> {
    println!("Running agdb benchmark");
    println!("---");

    let mut db = Database::new()?;
    setup_users(&mut db)?;
    let writers = start_post_writers(&mut db)?;

    db.stat()
}
